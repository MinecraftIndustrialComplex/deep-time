#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Where ocean crust's felsic and metamorphic rock lies on a checkpoint (the crust-template review,
tools/review/geo-template.sh): reads a `deeptime dump-fields` folder and prints, for the oceanic L1
cells, the share holding felsic plutonic, metamorphic and sedimentary rock and their mean
thickness by distance (in cells, x wrapping) from the nearest continental cell, and by crust age.

    python3 geo_template_fields.py <dump-fields folder>
"""
import array
import collections
import json
import os
import sys

d = sys.argv[1]
index = json.load(open(os.path.join(d, 'index.json')))


def field(name):
    e = index[name]
    a = array.array('f')
    with open(os.path.join(d, e['file']), 'rb') as f:
        a.frombytes(f.read())
    return a, e['w'], e['h']


def find(*parts):
    for n in index:
        if all(p in n for p in parts):
            return n
    return None


names = {
    'kind': find('crust_type') or find('kind'),
    'age': find('crust_age'),
    'fp': find('pool', 'felsic_plutonic'),
    'met': find('pool', 'metamorphic'),
    'sedy': find('pool', 'sedimentary_m'),
    'sed': find('pool', 'sediment_m'),
    'mv': find('pool', 'mafic_volcanic'),
}
print('fields:', json.dumps(names))
if not names['kind'] or not names['fp']:
    print('available:', ', '.join(sorted(index)))
    sys.exit(1)
kind, w, h = field(names['kind'])
f = {k: field(n)[0] for k, n in names.items() if n and k != 'kind'}
n = w * h
OCEANIC = 0
ocean = [kind[c] == OCEANIC and f['fp'][c] == f['fp'][c] for c in range(n)]
# Distance in cells from the nearest continental cell (4-neighbour BFS, x wraps, z clamps).
INF = 1 << 30
dist = [INF] * n
q = collections.deque()
for c in range(n):
    if kind[c] != OCEANIC and kind[c] == kind[c]:
        dist[c] = 0
        q.append(c)
while q:
    c = q.popleft()
    i, j = c % w, c // w
    for (a, b) in ((i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)):
        if b < 0 or b >= h:
            continue
        a %= w
        e = b * w + a
        if dist[e] > dist[c] + 1:
            dist[e] = dist[c] + 1
            q.append(e)


def table(key, label):
    rows = collections.defaultdict(lambda: [0, 0, 0, 0.0, 0.0, 0.0])
    for c in range(n):
        if not ocean[c]:
            continue
        b = key(c)
        if b is None:
            continue
        r = rows[b]
        r[0] += 1
        r[1] += f['fp'][c] > 50
        r[2] += f['met'][c] > 50
        r[3] += f['fp'][c]
        r[4] += f['met'][c]
        r[5] += f['sedy'][c] if 'sedy' in f else 0
    print(f'\n{label:>14} {"cells":>7} {"fp>50m":>7} {"met>50m":>8} {"fp m":>7} {"met m":>7} {"sedy m":>7}')
    for b in sorted(rows):
        r = rows[b]
        print(f'{b:>14} {r[0]:>7} {r[1] / r[0]:>7.2f} {r[2] / r[0]:>8.2f} {r[3] / r[0]:>7.0f} '
              f'{r[4] / r[0]:>7.0f} {r[5] / r[0]:>7.0f}')


EDGES = [1, 2, 3, 5, 8, 13, 21, 34, 1 << 29]


def by_dist(c):
    for k, e in enumerate(EDGES):
        if dist[c] <= e:
            return f'{k}: <= {e}' if e < 1 << 29 else f'{k}: farther'
    return None


BANDS = [(0, 20), (20, 65), (65, 150), (150, 1e9)]


def by_age(c):
    if 'age' not in f:
        return None
    a = f['age'][c]
    for lo, hi in BANDS:
        if lo <= a < hi:
            return f'{lo}-{hi if hi < 1e8 else ""}'
    return None


table(by_dist, 'cells to land')
table(by_age, 'age Myr')
