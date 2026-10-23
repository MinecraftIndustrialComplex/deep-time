# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's comparison table: per-measure medians, two-sided Mann-Whitney U against a base arm, the
criteria met per run, in the markdown shape of docs/science/plates.md ("V1 item 1's seven measures").
Standard library only (the VM's scipy is used by the tests only, to check `mwu`)."""
import functools
import math
import statistics

MINUS = "−"


def _ranks(values):
    order = sorted(range(len(values)), key=lambda i: values[i])
    ranks = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for k in range(i, j + 1):
            ranks[order[k]] = (i + j) / 2.0 + 1.0
        i = j + 1
    return ranks


@functools.lru_cache(maxsize=None)
def _u_counts(m, n):
    """counts[u]: the number of arrangements of m + n distinct values whose U statistic is u."""
    if m == 0 or n == 0:
        return (1,)
    a, b = _u_counts(m - 1, n), _u_counts(m, n - 1)
    out = [0] * (m * n + 1)
    for u, c in enumerate(a):
        out[u + n] += c  # the largest value belongs to the first sample: it beats all n of the second
    for u, c in enumerate(b):
        out[u] += c
    return tuple(out)


def mwu(x, y):
    """Two-sided Mann-Whitney U p-value, as scipy.stats.mannwhitneyu(x, y) gives it by default: exact
    when the smaller sample has 8 values or fewer and there are no ties, else the normal approximation with
    the tie and the continuity corrections. None when a sample is empty."""
    n1, n2 = len(x), len(y)
    if n1 == 0 or n2 == 0:
        return None
    ranks = _ranks(list(x) + list(y))
    u1 = sum(ranks[:n1]) - n1 * (n1 + 1) / 2.0
    u = max(u1, n1 * n2 - u1)
    n = n1 + n2
    tied = len(set(x) | set(y)) < n
    if not tied and min(n1, n2) <= 8:
        counts = _u_counts(n1, n2)
        tail = sum(counts[int(round(u)):]) / sum(counts)
        return min(1.0, 2.0 * tail)
    counts = {}
    for v in list(x) + list(y):
        counts[v] = counts.get(v, 0) + 1
    tie_term = sum(t ** 3 - t for t in counts.values()) / (n * (n - 1))
    s = math.sqrt(n1 * n2 / 12.0 * ((n + 1) - tie_term))
    if s == 0:
        return 1.0
    z = (u - n1 * n2 / 2.0 - 0.5) / s
    return min(1.0, max(0.0, math.erfc(z / math.sqrt(2.0))))


def fmt_p(p):
    if p is None:
        return ""
    if p < 0.001:
        return "< 0.001"
    if p >= 0.995:
        return "1.0"
    return f"{p:.2f}" if p >= 0.1 else f"{p:.3f}"


def fmt_num(v, digits, signed=False):
    s = f"{abs(v):.{digits}f}"
    if v < 0 and float(s) != 0:
        return MINUS + s
    return ("+" if signed and float(s) != 0 else "") + s


def fmt_target(t):
    if not t:
        return ""
    lo, hi = t
    if lo is not None and hi is not None:
        return f"{lo:g} to {hi:g}"
    if lo is not None:
        return f"≥ {lo:g}"
    return f"≤ {hi:g}"


def met(value, target):
    if value is None or not target:
        return False
    lo, hi = target
    return (lo is None or value >= lo) and (hi is None or value <= hi)


def column_values(runs, key):
    return [r[key] for r in runs if r.get(key) is not None and not (isinstance(r[key], float) and math.isnan(r[key]))]


def criteria_counts(runs, spec):
    targets = {c["key"]: c.get("target") for c in spec["columns"] if "key" in c}
    keys = spec.get("criteria", [])
    return [sum(met(r.get(k), targets.get(k)) for k in keys) for r in runs]


def summarise(spec, base, arms):
    """{arm: {column key: {"n", "median", "p"}}}, plus "criteria" and the pair columns. `base` and each
    arm in `arms` are (name, [per-run value dicts])."""
    out = {}
    base_runs = base[1]
    for idx, (name, runs) in enumerate([base] + arms):
        is_base = idx == 0
        cells = {}
        for c in spec["columns"]:
            if c.get("special") == "criteria":
                mine, theirs = criteria_counts(runs, spec), criteria_counts(base_runs, spec)
                cells["criteria"] = {"n": len(mine), "median": statistics.median(mine) if mine else None,
                                     "mean": statistics.mean(mine) if mine else None,
                                     "p": None if is_base else mwu(theirs, mine), "runs": mine}
                continue
            key = c["key"]
            vals = column_values(runs, key)
            cell = {"n": len(vals), "median": statistics.median(vals) if vals else None, "values": vals}
            if c.get("pair"):
                v2 = column_values(runs, c["key2"])
                cell["median2"] = statistics.median(v2) if v2 else None
            else:
                cell["p"] = None if is_base else mwu(column_values(base_runs, key), vals)
            cells[key] = cell
        out[name] = cells
    return out


def render(spec, base, arms, name="v1census"):
    """The markdown table (and a note on the sample sizes)."""
    summary = summarise(spec, base, arms)
    cols = spec["columns"]
    lines = ["| arm | " + " | ".join(c["label"] for c in cols) + " |", "|---|" + "---|" * len(cols)]
    lines.append("| target | " + " | ".join(
        (f"of {len(spec.get('criteria', []))}" if c.get("special") == "criteria" else fmt_target(c.get("target"))) for c in cols) + " |")
    for arm_name, _ in [base] + arms:
        cells = []
        for c in cols:
            if c.get("special") == "criteria":
                s = summary[arm_name]["criteria"]
                if s["median"] is None:
                    cells.append("")
                    continue
                txt = f"{s['median']:g}, {s['mean']:.2f}"
                cells.append(txt + (f" ({fmt_p(s['p'])})" if s["p"] is not None else ""))
                continue
            s = summary[arm_name][c["key"]]
            if s["median"] is None:
                cells.append("")
            elif c.get("pair"):
                second = "" if s["median2"] is None else fmt_num(s["median2"], c.get("digits", 3))
                cells.append(f"{fmt_num(s['median'], c.get('digits', 3))} / {second}")
            else:
                txt = fmt_num(s["median"], c.get("digits", 3), c.get("signed", False))
                cells.append(txt + (f" ({fmt_p(s['p'])})" if s["p"] is not None else ""))
        lines.append(f"| {arm_name} | " + " | ".join(cells) + " |")
    sizes = ", ".join(f"{n} {len(r)}" for n, r in [base] + arms)
    lines.append("")
    lines.append(f"Measure `{name}`: median over the runs of each arm ({sizes} runs), the two-sided Mann-Whitney U p "
                 f"against {base[0]} in brackets. Criteria met per run: of the {len(spec.get('criteria', []))} targets above.")
    return "\n".join(lines) + "\n", summary
