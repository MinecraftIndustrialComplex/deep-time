#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The crust-template and relief-lift review's report (tools/review/geo-template.sh report).

Reads every censused run under RUNS (`<tag>-<start>-s<seed>` folders and the census files
`geo-template.sh census` wrote beside them) and writes OUT/report.md, OUT/report.json and the
section figures OUT/<line>-<start>-s<seed>.png (one panel per tag).

Ocean crust census (the triage's format): the sea columns of the 208-block grid whose crust has an
age (`meta/crust_age_myr`), by age band: the mean thickness of each rock group and the share of
columns holding any of it, and the median depth from the sea floor to the first peridotite.
"""
import collections
import glob
import json
import os
import statistics
import sys

M_PER_BLOCK = 35.0
SED = {'conglomerate', 'diamictite', 'sandstone', 'red_bed', 'mudstone', 'shale', 'black_shale',
       'limestone', 'dolostone', 'chalk', 'chert', 'travertine', 'halite', 'gypsum', 'anhydrite',
       'coal', 'ironstone', 'banded_iron_formation', 'phosphorite', 'soil', 'saprolite',
       'laterite', 'bauxite', 'gravel', 'sand', 'clay', 'mud', 'till', 'loess', 'volcanic_ash',
       'tuff'}
LAVA = {'basalt', 'andesite', 'rhyolite', 'obsidian', 'scoria', 'pumice'}
DIKES = {'greenstone'}
GAB = {'gabbro', 'anorthosite', 'amphibolite'}
FEL = {'granite', 'granodiorite', 'diorite', 'pegmatite', 'greisen'}
MET = {'gneiss', 'schist', 'phyllite', 'slate', 'quartzite', 'marble', 'blueschist', 'eclogite',
       'hornfels', 'skarn'}
PER = {'peridotite', 'serpentinite'}
GROUPS = [('sed', SED), ('lava', LAVA), ('dikes', DIKES), ('gabbro', GAB), ('granitoid', FEL),
          ('meta', MET), ('peridotite', PER)]
SHOW = ['sed', 'lava', 'dikes', 'gabbro', 'granitoid', 'meta']
BANDS = [(0, 20), (20, 65), (65, 150), (150, 1e9)]
COLOURS = {'sed': '#e3c77a', 'lava': '#4a4a4a', 'dikes': '#5f8f5a', 'gabbro': '#2f5f8a',
           'granitoid': '#e58fa0', 'meta': '#9b6fb5', 'peridotite': '#7aa35c', 'other': '#bbbbbb'}


def grp(c):
    for n, s in GROUPS:
        if c in s:
            return n
    return 'other'


def load(path):
    rows = []
    if not os.path.exists(path):
        return rows
    for line in open(path):
        line = line.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def nearest(d, name):
    return ((d.get('layers') or {}).get(name) or {}).get('nearest')


def row_of(d):
    return dict(x=d['x'], z=d['z'], top_y=d['top_y'], kind=(d.get('hydro') or {}).get('kind'),
                age=nearest(d, 'meta/crust_age_myr'), rock=d.get('rock') or [],
                strata=[(s['top'], s['bottom'], s['class']) for s in d.get('strata') or []])


def stack(r):
    t = collections.Counter()
    first = {}
    for top, bot, c in r['rock']:
        if c is None:
            continue
        g = grp(c)
        t[g] += top - bot + 1
        first.setdefault(g, top)
    moho = first.get('peridotite')
    return t, (r['top_y'] - moho) if moho is not None else None


def census(rows):
    sea = [r for r in rows if r['kind'] == 'sea']
    ocean = [r for r in sea if r['age'] is not None and r['age'] >= 0]
    out = {'columns': len(rows), 'sea': len(sea), 'ocean': len(ocean),
           'land_share_of_grid': (1 - len(sea) / len(rows)) if rows else None, 'bands': []}

    def table(sel, label):
        n = len(sel)
        if n == 0:
            return None
        tot, has, crust = collections.Counter(), collections.Counter(), []
        for r in sel:
            t, c = stack(r)
            tot.update(t)
            for g, _ in GROUPS:
                if t[g] > 0:
                    has[g] += 1
            if c is not None:
                crust.append(c)
        return {'label': label, 'n': n,
                'mean_m': {g: tot[g] / n * M_PER_BLOCK for g, _ in GROUPS},
                'share': {g: has[g] / n for g, _ in GROUPS},
                'seabed_to_peridotite_m': statistics.median(crust) * M_PER_BLOCK if crust else None}

    for lo, hi in BANDS:
        t = table([r for r in ocean if lo <= r['age'] < hi],
                  f"{lo}-{hi if hi < 1e8 else ''} Myr")
        if t:
            out['bands'].append(t)
    t = table(ocean, 'all ages')
    if t:
        out['bands'].append(t)
    out['continental_sea'] = table([r for r in sea if r['age'] is None or r['age'] < 0], 'continental crust, sea')
    out['land'] = table([r for r in rows if r['kind'] != 'sea'], 'land')
    return out


def bodies_check(rows, deposits):
    """Bodies whose centre stands above the ground at its column, and their depth below it."""
    if not rows or not deposits:
        return None
    cen = {}
    for b in deposits.get('bodies', []):
        body = b.get('body') or b
        c = body.get('center')
        if c:
            cen.setdefault((round(c[0]), round(c[2])), []).append(c[1])
    above, depth = 0, []
    n = 0
    for r in rows:
        for y in cen.get((r['x'], r['z']), []):
            n += 1
            d = r['top_y'] - y
            depth.append(d)
            above += d < 0
    if n == 0:
        return None
    return {'checked': n, 'centre_above_ground': above / n,
            'median_depth_blocks': statistics.median(depth)}


def deposit_counts(deposits):
    if not deposits:
        return None
    sites = collections.Counter(s.get('deposit_type', '?') for s in deposits.get('sites', []))
    return {'sites': sum(sites.values()), 'bodies': len(deposits.get('bodies', [])),
            'by_type': dict(sites.most_common())}


def runs(root):
    out = {}
    for log in sorted(glob.glob(os.path.join(root, '*.log'))):
        name = os.path.basename(log)[:-4]
        if not os.path.exists(os.path.join(root, name + '-grid.jsonl')):
            continue
        tag, rest = name.split('-', 1)
        start, seed = rest.rsplit('-s', 1)
        out[name] = (tag, start, int(seed))
    return out


def fmt_band(b):
    cells = [f"{b['mean_m'][g]:.0f} ({b['share'][g] * 100:.0f}%)" for g in SHOW]
    moho = b['seabed_to_peridotite_m']
    return '| ' + ' | '.join([b['label'], str(b['n'])] + cells +
                             [f"{moho / 1000:.1f}" if moho is not None else '-']) + ' |'


def section_plot(plt, panels, path, title):
    fig, axes = plt.subplots(len(panels), 1, figsize=(12, 3.2 * len(panels)), squeeze=False)
    for ax, (label, rows) in zip(axes[:, 0], panels):
        for i, r in enumerate(rows):
            for top, bot, c in r['rock']:
                if c is None or top < -520:
                    continue
                ax.bar(i, top - bot + 1, bottom=bot, width=1.0, color=COLOURS[grp(c)],
                       linewidth=0)
        ax.plot(range(len(rows)), [r['top_y'] for r in rows], color='black', linewidth=0.8)
        ax.axhline(63, color='#3a7bd5', linewidth=0.6, linestyle='--')
        ax.set_xlim(-0.5, len(rows) - 0.5)
        lo = min(r['top_y'] for r in rows) - 200
        hi = max(r['top_y'] for r in rows) + 20
        ax.set_ylim(max(-512, lo), hi)
        ax.set_title(f"{title}: {label}", fontsize=9)
        ax.set_ylabel('block y')
    handles = [plt.Rectangle((0, 0), 1, 1, color=COLOURS[g]) for g in COLOURS]
    axes[0, 0].legend(handles, list(COLOURS), fontsize=7, ncol=8, loc='upper right')
    axes[-1, 0].set_xlabel('point along the line')
    fig.tight_layout()
    fig.savefig(path, dpi=110)
    plt.close(fig)


def main():
    root, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    report = {}
    for name, (tag, start, seed) in runs(root).items():
        base = os.path.join(root, name)
        grid = [row_of(d) for d in load(base + '-grid.jsonl') if 'err' not in d]
        man = json.load(open(os.path.join(base, 'manifest.json')))
        bake = (man.get('notes') or {}).get('bake') or {}
        strata = bake.get('strata') or {}
        deposits = None
        if os.path.exists(base + '-deposits.json'):
            deposits = json.load(open(base + '-deposits.json'))
        bodies = [row_of(d) for d in load(base + '-bodies.jsonl') if 'err' not in d]
        proj = bake.get('projection') or {}
        report[name] = {
            'tag': tag, 'start': start, 'seed': seed,
            'census': census(grid),
            'mountain_fill': strata.get('mountain_fill'),
            'lift': strata.get('lift'),
            'top_raised_cells': strata.get('top_raised_cells'),
            'land': {k: v for k, v in proj.items() if 'land' in k},
            'deposits': deposit_counts(deposits),
            'bodies': bodies_check(bodies, deposits),
        }
    json.dump(report, open(os.path.join(out, 'report.json'), 'w'), indent=1)

    md = ['# Crust template and relief lift review', '']
    head = ('| age band | n | ' + ' | '.join(f'{g} m (share)' for g in SHOW) +
            ' | seabed to peridotite km |')
    sep = '|' + '---|' * (len(SHOW) + 3)
    for name in sorted(report):
        r = report[name]
        c = r['census']
        md += [f"## {name}", '',
               f"columns {c['columns']}, sea {c['sea']}, ocean (aged crust) {c['ocean']}, "
               f"land share of grid {c['land_share_of_grid']:.3f}", '', head, sep]
        md += [fmt_band(b) for b in c['bands']]
        if c['continental_sea']:
            md.append(fmt_band(c['continental_sea']))
        md.append('')
        mf = r['mountain_fill']
        if mf:
            for row in mf.get('by_relief', []):
                md.append(f"mountain fill, relief >= {row['min_relief_blocks']:.0f} blocks: "
                          f"{row['cells']} cells, top unit fills most {row['top_unit_fills_most']:.3f}, "
                          f"mean share {row['mean_share_filled']:.3f}")
        if r['lift']:
            md.append(f"lift: {json.dumps({k: v for k, v in r['lift'].items() if k != 'rule'})}")
        md.append(f"land: {json.dumps(r['land'])}")
        if r['deposits']:
            d = r['deposits']
            md.append(f"deposits: {d['sites']} sites, {d['bodies']} bodies; "
                      + ', '.join(f'{k} {v}' for k, v in list(d['by_type'].items())[:12]))
        if r['bodies']:
            md.append(f"bodies: {json.dumps(r['bodies'])}")
        md.append('')
    open(os.path.join(out, 'report.md'), 'w').write('\n'.join(md) + '\n')

    try:
        import matplotlib
        matplotlib.use('Agg')
        import matplotlib.pyplot as plt
    except ImportError:
        print('no matplotlib: sections skipped')
        return
    keys = sorted({(r['start'], r['seed']) for r in report.values()})
    for start, seed in keys:
        for line in ('line1', 'line2'):
            panels = []
            for name in sorted(report):
                r = report[name]
                if (r['start'], r['seed']) != (start, seed):
                    continue
                rows = [row_of(d) for d in load(os.path.join(root, name) + f'-{line}.jsonl')
                        if 'err' not in d]
                if rows:
                    panels.append((r['tag'], rows))
            if panels:
                section_plot(plt, panels, os.path.join(out, f'{line}-{start}-s{seed}.png'),
                             f'{line} {start} s{seed}')
    print('report', os.path.join(out, 'report.md'))


if __name__ == '__main__':
    main()
