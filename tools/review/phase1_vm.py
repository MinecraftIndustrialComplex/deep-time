#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The test-VM half of the Phase 1 exit suite (plan §10.3 "Phase 1 exit test", items 3 and 4).

    tools/review/phase1_vm.py pick     --world smoke/mic/server/planet --out review/out/phase1/plan.json
    tools/review/phase1_vm.py serve    --profile mic --plan review/out/phase1/plan.json
    tools/review/phase1_vm.py geology  --world smoke/mic/server/planet --plan review/out/phase1/plan.json
                                       --rcon smoke/mic/rcon.jsonl --out review/out/phase1/geology
    tools/review/phase1_vm.py compat   --plan … --rcon smoke/mic/rcon.jsonl

Run from the repo root on the VM (tools/review/phase1.py calls them through tools/remote/mac.sh). Nothing
here is heavy except what `serve` execs (tools/smoke/run.py admits itself) and `geology`'s deeptime-inspect
calls (a few chunks each). Standard library only.

`pick` reads a first-start world's sites (deeptime dump-deposits), chooses the porphyry nearest to spawn
with its gossan, its placer child, three more porphyries of other tonnages, petroleum traps and empty
chunks, and writes the plan: the Chunky squares and the RCON probes a second server run executes. `serve`
runs that second boot (the world stays; no --fresh). `geology` and `compat` read the probes it recorded
(smoke/<profile>/rcon.jsonl) and `deeptime-inspect`, and print one line per check:

    @check PASS|FAIL <title> :: <evidence>

which phase1.py turns into rows. A check whose precondition is missing (no site, no essentials list) FAILs
with the reason: the suite is correct when the work behind it is unfinished.
"""

import argparse
import json
import math
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DEEPTIME = os.path.join(REPO, "target", "release", "deeptime")
INSPECT = os.path.join(REPO, "target", "release", "deeptime-inspect")

PORPHYRY = "porphyry_cu_mo_au"
PLACER = "placer_au"
TRAP = "petroleum_trap"
# Chunks around a site's ore chunk that the COE/CDG probes cover (a body may straddle a chunk edge).
RING = 1


def check(ok, title, evidence):
    print(f"@check {'PASS' if ok else 'FAIL'} {title} :: {evidence}", flush=True)
    return ok


def short(t):
    return t.split(":")[-1].split("/")[-1]


def wdist(c, x0, z0, x1, z1):
    dx = (x1 - x0) % c
    dx = min(dx, c - dx)
    return math.hypot(dx, z1 - z0)


def chunk_of(v):
    return v // 16


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, cwd=REPO, **kw)


# ---------------------------------------------------------------------------------------------
# pick
# ---------------------------------------------------------------------------------------------
def cmd_pick(a):
    wd = os.path.join(a.world, "deeptime")
    dump = a.out + ".dump.json"
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    r = run([DEEPTIME, "dump-deposits", "--world", wd, "--out", dump])
    if r.returncode:
        print(r.stdout + r.stderr)
        sys.exit("pick: dump-deposits failed")
    d = json.load(open(dump))
    c = d["circumference"]
    sites = d["sites"]
    bodies = [b["body"] for b in d["bodies"]]
    by_site = {}
    for b in bodies:
        by_site.setdefault(b["site"], []).append(b)
    manifest = json.load(open(os.path.join(wd, "manifest.json")))
    cands_path = os.path.join(wd, "spawn", "candidates.json")
    cands = json.load(open(cands_path)) if os.path.exists(cands_path) else {"candidates": []}
    cl = cands.get("candidates", [])
    # The game takes the dry candidate with the fewest unmet essentials, file order breaking ties.
    spawn = min(enumerate(cl), key=lambda it: (it[1].get("unmet", 0), it[0]))[1] if cl else {"x": 0, "z": 0}
    sx, sz = spawn["x"], spawn["z"]
    ranked = cands.get("ranked_by", [])
    reach = next((r["max_blocks_from_spawn"] for r in ranked if "copper" in r["material"]), None)

    def of(t):
        return [s for s in sites if short(s["deposit_type"]) == t]

    def tonnage(s):
        return max((p["tonnage_t"] for p in s["products"]), default=0.0)

    def dist(s):
        return wdist(c, sx, sz, s["center"][0], s["center"][2])

    por = sorted(of(PORPHYRY), key=dist)
    plan = {"circumference": c, "spawn": {"x": sx, "z": sz}, "essentials_reach": reach,
            "essentials": ranked, "manifest_essentials": (manifest.get("notes") or {}).get("essentials"),
            "counts": {short(t): sum(1 for s in sites if short(s["deposit_type"]) == short(t))
                       for t in {s["deposit_type"] for s in sites}},
            "porphyry": None, "gossan": None, "placer": None, "others": [], "traps": [], "empty": [],
            "chunky": [], "rcon": []}
    if por:
        p = por[0]
        plan["porphyry"] = {"site": p["id"], "center": p["center"], "aabb": p["aabb"], "tonnage_t": tonnage(p),
                            "distance_to_spawn": round(dist(p), 1),
                            "bodies": [{k: b.get(k) for k in ("id", "role", "shape", "center", "axes", "aabb", "clip_top_y", "rock_class")}
                                       for b in by_site.get(p["id"], [])]}
        g = next((b for b in by_site.get(p["id"], []) if b["role"] == "gossan"), None)
        plan["gossan"] = g
        kids = [s for s in of(PLACER) if s.get("parent") == p["id"]]
        if kids:
            k = min(kids, key=lambda s: wdist(c, p["center"][0], p["center"][2], s["center"][0], s["center"][2]))
            plan["placer"] = {"site": k["id"], "center": k["center"], "aabb": k["aabb"], "tonnage_t": tonnage(k),
                              "distance_to_parent": round(wdist(c, p["center"][0], p["center"][2], k["center"][0], k["center"][2]), 1)}
        # Three more porphyries across the tonnage range (for "yield follows tonnage").
        rest = sorted((s for s in por[1:] if tonnage(s) > 0), key=tonnage)
        picks = [rest[int(i * (len(rest) - 1) / 2)] for i in range(3)] if len(rest) >= 3 else rest
        plan["others"] = [{"site": s["id"], "center": s["center"], "tonnage_t": tonnage(s)} for s in picks]
    traps = sorted(of(TRAP), key=tonnage, reverse=True)
    plan["traps"] = [{"site": s["id"], "center": s["center"], "tonnage_t": tonnage(s)} for s in traps[:3]]
    # Empty chunks: the spawn candidates (dry land) farthest from every site, at least 600 blocks away.
    far = []
    for cd in cl:
        dmin = min((wdist(c, cd["x"], cd["z"], s["center"][0], s["center"][2]) for s in sites), default=1e9)
        far.append((dmin, cd))
    far.sort(key=lambda t: -t[0])
    plan["empty"] = [{"x": cd["x"], "z": cd["z"], "nearest_site": round(dm, 1)} for dm, cd in far[:3] if dm >= 600]
    if not plan["empty"] and far:
        plan["empty"] = [{"x": cd["x"], "z": cd["z"], "nearest_site": round(dm, 1)} for dm, cd in far[:1]]

    chunky = []
    rc = []

    def ring_probes(kind, cx, cz):
        for dx in range(-RING, RING + 1):
            for dz in range(-RING, RING + 1):
                rc.append(f"deeptime probe {kind} {cx + dx} {cz + dz}")

    if plan["porphyry"]:
        x, y, z = (int(v) for v in plan["porphyry"]["center"])
        chunky.append([x, z, a.radius])
        rc += [f"forceload add {x} {z}", "sleep 4",
               f"deeptime sites near {x} {z} 600", f"deeptime why {plan['porphyry']['site']}",
               f"deeptime probe site {x} {y} {z}"]
        if plan["gossan"]:
            gx, gy, gz = (int(v) for v in plan["gossan"]["center"])
            rc += [f"forceload add {gx} {gz}", "sleep 3", f"deeptime probe site {gx} {gy} {gz}"]
            top = plan["gossan"].get("clip_top_y")
            if top is not None:
                rc.append(f"deeptime probe site {gx} {int(top) - 1} {gz}")
        ring_probes("coe", chunk_of(x), chunk_of(z))
    if plan["placer"]:
        x, y, z = (int(v) for v in plan["placer"]["center"])
        chunky.append([x, z, 96])
        rc += [f"forceload add {x} {z}", "sleep 3", f"deeptime sites near {x} {z} 400", f"deeptime probe site {x} {y} {z}"]
        ring_probes("coe", chunk_of(x), chunk_of(z))
    for o in plan["others"]:
        ring_probes("coe", chunk_of(int(o["center"][0])), chunk_of(int(o["center"][2])))
    for e in plan["empty"]:
        ring_probes("coe", chunk_of(e["x"]), chunk_of(e["z"]))
        ring_probes("cdg", chunk_of(e["x"]), chunk_of(e["z"]))
    for t in plan["traps"]:
        ring_probes("cdg", chunk_of(int(t["center"][0])), chunk_of(int(t["center"][2])))
    rc += ["deeptime probe claims"]
    plan["chunky"] = chunky
    plan["rcon"] = rc
    json.dump(plan, open(a.out, "w"), indent=1)
    print(json.dumps({k: plan[k] for k in ("circumference", "spawn", "essentials_reach", "counts")}))
    print(f"pick: porphyry {plan['porphyry']['site'] if plan['porphyry'] else 'none'}, placer "
          f"{plan['placer']['site'] if plan['placer'] else 'none'}, gossan {'yes' if plan['gossan'] else 'no'}, "
          f"{len(plan['others'])} other porphyries, {len(plan['traps'])} traps, {len(plan['empty'])} empty chunks, "
          f"{len(rc)} probes, {len(chunky)} Chunky squares -> {a.out}")


# ---------------------------------------------------------------------------------------------
# serve: the second boot, exec'd into tools/smoke/run.py
# ---------------------------------------------------------------------------------------------
def cmd_serve(a):
    plan = json.load(open(a.plan))
    args = [sys.executable, os.path.join(REPO, "tools", "smoke", "run.py"), "--profile", a.profile, "--timeout", str(a.timeout)]
    for x, z, r in plan["chunky"]:
        args += ["--chunky-at", f"{x},{z},{r}"]
    for cmd in plan["rcon"]:
        args += ["--rcon", cmd]
    os.chdir(REPO)
    os.execv(sys.executable, args)


# ---------------------------------------------------------------------------------------------
# geology (item 3) and compat (item 4)
# ---------------------------------------------------------------------------------------------
def load_rcon(path):
    rows = []
    if os.path.exists(path):
        for line in open(path):
            try:
                rows.append(json.loads(line))
            except ValueError:
                pass
    return rows


def probes(rows, kind):
    return [r["probe"] for r in rows if r.get("probe") and r["probe"].get("probe") == kind]


def blocks_in(world, x0, z0, x1, z1):
    r = run([INSPECT, "blocks", "--world", world, f"--area={x0},{z0},{x1 + 1},{z1 + 1}"])
    try:
        doc = json.loads(r.stdout)
    except ValueError:
        return None
    return {k: v for k, v in (doc.get("blocks", doc) if isinstance(doc, dict) else {}).items() if isinstance(v, (int, float))}


def count(blocks, needles):
    return sum(v for k, v in (blocks or {}).items() if any(n in k for n in needles))


def cmd_geology(a):
    plan = json.load(open(a.plan))
    rows = load_rcon(a.rcon)
    os.makedirs(a.out, exist_ok=True)
    por, gos, pla = plan["porphyry"], plan["gossan"], plan["placer"]
    reach = plan.get("essentials_reach")

    # 3a
    if not por:
        check(False, "a porphyry site exists on the planet", f"no {PORPHYRY} site in the bake (counts {plan['counts']})")
    elif reach is None:
        check(False, "a porphyry site within the essentials reach of spawn",
              f"the preset names no copper essential (no max_blocks_from_spawn in spawn/candidates.json); nearest porphyry "
              f"{por['distance_to_spawn']:.0f} blocks from spawn")
    else:
        check(por["distance_to_spawn"] <= reach, "a porphyry site within the essentials reach of spawn",
              f"site {por['site']} is {por['distance_to_spawn']:.0f} blocks from spawn {plan['spawn']['x']},{plan['spawn']['z']}; reach {reach}")
    if not por:
        for t in ("`probe site` returns the body", "copper ore inside the body", "a gossan where the body is exposed",
                  "placer gold downstream, a child of the porphyry"):
            check(False, t, "no porphyry site")
        return

    # 3b: probe site at the ore body's centre
    ore = next((b for b in por["bodies"] if b["role"] == "ore"), None)
    ps = [p for p in probes(rows, "site") if por["site"] == (p.get("site") or {}).get("id")]
    ok_b = False
    ev = "no `probe site` answer for the porphyry (did the second boot run?)"
    if ps:
        p0 = ps[0]
        roles = sorted({b.get("role") for b in p0.get("bodies", [])})
        ok_b = bool(p0.get("site")) and "ore" in roles and p0.get("deepTimeLevel") is True
        ev = f"site {p0.get('site', {}).get('id')} ({short(p0.get('site', {}).get('type', ''))}), bodies here {roles}, ore_here {[b.get('ore_here') for b in p0.get('bodies', [])]}, block {p0.get('block')}"
    check(ok_b, "`probe site` returns the porphyry's body", ev)

    # 3c: copper ore inside the body (deeptime-inspect blocks over the body's footprint against a control box)
    if ore:
        lo, hi = ore["aabb"]
        inside = blocks_in(a.world, lo[0], lo[2], hi[0], hi[2])
        w = hi[0] - lo[0] + 1
        cx0 = lo[0] + 3 * w + 64
        control = blocks_in(a.world, cx0, lo[2], cx0 + w - 1, hi[2])
        cu = ("copper_ore", "raw_copper_block")
        n_in, n_out = count(inside, cu), count(control, cu)
        ore_here = [b.get("ore_here") for p in ps for b in p.get("bodies", []) if b.get("role") == "ore"]
        ok_c = inside is not None and n_in >= 20 and n_in >= 5 * max(n_out, 1)
        check(ok_c, "`deeptime-inspect` shows copper ore inside the body",
              f"{n_in} copper ore blocks in the body's footprint {w}x{hi[2] - lo[2] + 1} (control box beside it: {n_out}); probe ore_here {ore_here}")
        # the picture: ore map and a section through the body
        run([INSPECT, "ore", "--world", a.world, "--out", a.out, f"--area={lo[0] - 24},{lo[2] - 24},{hi[0] + 25},{hi[2] + 25}"])
        zc = int(ore["center"][2])
        sec = run([INSPECT, "section", "--world", a.world, "--out", a.out, "--axis", "z", "--at", str(zc),
                   "--from", str(lo[0] - 40), "--to", str(hi[0] + 40), "--scale", "4"])
        print("section:", (sec.stdout or sec.stderr).strip().splitlines()[-1:] or "")
    else:
        check(False, "`deeptime-inspect` shows copper ore inside the body", f"site {por['site']} has no `ore` body")

    # 3d: gossan where the body is exposed
    if gos:
        gp = [b for p in probes(rows, "site") for b in p.get("bodies", []) if b.get("role") == "gossan" and b.get("site") == por["site"]]
        blocks = [p.get("block") for p in probes(rows, "site") if any(b.get("role") == "gossan" for b in p.get("bodies", []))]
        glo, ghi = gos["aabb"]
        gb = blocks_in(a.world, glo[0], glo[2], ghi[0], ghi[2])
        iron = count(gb, ("ochrum", "ironstone", "gossan", "limonite"))
        ok_d = bool(gp) and (iron > 0 or any(b and re.search(r"ochrum|ironstone|gossan|limonite", b) for b in blocks))
        check(ok_d, "a gossan where the body is exposed",
              f"gossan body {gos['id']} ({gos.get('rock_class')}, clip_top_y {gos.get('clip_top_y')}); probe found it at {len(gp)} point(s), "
              f"blocks there {blocks[:2]}; ironstone-class blocks in its footprint: {iron}")
    else:
        check(False, "a gossan where the body is exposed",
              f"site {por['site']} has no gossan body (a body that is not exposed has none; bodies: {[b['role'] for b in por['bodies']]})")

    # 3e: placer gold, a child, downstream
    if pla:
        down = pla["center"][1] < por["center"][1]
        lo, hi = pla["aabb"]
        pb = blocks_in(a.world, lo[0], lo[2], hi[0], hi[2])
        gold = count(pb, ("gold_ore", "raw_gold_block"))
        check(down and gold > 0, "placer gold downstream, a child of the porphyry",
              f"site {pla['site']} (parent {por['site']}) {pla['distance_to_parent']:.0f} blocks away, centre y {pla['center'][1]} vs porphyry y "
              f"{por['center'][1]} ({'lower' if down else 'NOT lower'}); {gold} gold ore blocks in its footprint")
    else:
        check(False, "placer gold downstream, a child of the porphyry", f"no {PLACER} site has parent {por['site']}")


def cmd_compat(a):
    plan = json.load(open(a.plan))
    rows = load_rcon(a.rcon)
    coe = probes(rows, "coe")
    cdg = probes(rows, "cdg")
    por = plan["porphyry"]
    if not coe:
        for t in ("COE reports the copper vein over the porphyry", "COE claims nothing where there is no site",
                  "COE's yield follows tonnage"):
            check(False, t, "no `probe coe` answers (second boot did not run)")
    else:
        # the vein over the porphyry
        site_ids = {por["site"]} if por else set()
        over = [p for p in coe if p.get("site_vein") and p["site_vein"]["site"] in site_ids]
        agree = [p for p in over if p.get("agrees") is True]
        veins = sorted({short(p["site_vein"]["vein"]) for p in over})
        copper = [v for v in veins if "copper" in v or "cu" in v]
        check(bool(over) and len(agree) == len(over) and bool(copper), "COE reports the copper vein over the porphyry",
              f"{len(over)} chunk(s) over site {por['site'] if por else '?'} carry its vein {veins}; COE agrees in {len(agree)}; COE installed: "
              f"{coe[0].get('coe_installed')}")
        # nothing claimed where there is no site
        empties = {(chunk_of(e["x"]) + dx, chunk_of(e["z"]) + dz) for e in plan["empty"] for dx in range(-RING, RING + 1) for dz in range(-RING, RING + 1)}
        none = [p for p in coe if tuple(p["chunk"]) in empties]
        claimed = set(re.sub(r".*/", "", v) for p in none for v in p.get("claimed_veins", []))
        bad = []
        for p in none:
            co = p.get("coe") or {}
            vein = co.get("vein")
            if p.get("site_vein") or (vein and short(vein) in {short(v) for v in p.get("claimed_veins", [])}):
                bad.append(tuple(p["chunk"]))
        check(bool(none) and not bad and all(p.get("agrees") is not False for p in none),
              "COE claims nothing where there is no site",
              f"{len(none)} chunks around {len(plan['empty'])} spots (nearest site {[e['nearest_site'] for e in plan['empty']]} blocks) with a claimed vein: "
              f"{len(bad)} {bad[:3]}; claimed veins {sorted(claimed)[:6]}")
        # yield follows tonnage: the best chunk of each of the sampled porphyries
        samples = []
        sites = ([por] if por else []) + ([{"site": o["site"], "center": o["center"], "tonnage_t": o["tonnage_t"]} for o in plan["others"]])
        for s in sites:
            ts = [p for p in coe if p.get("site_vein") and p["site_vein"]["site"] == s["site"] and p.get("coe")]
            if ts:
                best = max(ts, key=lambda p: p["site_vein"]["ore_blocks_in_chunk"])
                samples.append((s["tonnage_t"], best["coe"]["amount"], s["site"]))
        if len(samples) >= 3:
            order_t = [i for i, _ in sorted(enumerate(samples), key=lambda t: t[1][0])]
            order_a = [i for i, _ in sorted(enumerate(samples), key=lambda t: t[1][1])]
            rt = {i: r for r, i in enumerate(order_t)}
            ra = {i: r for r, i in enumerate(order_a)}
            n = len(samples)
            rho = 1 - 6 * sum((rt[i] - ra[i]) ** 2 for i in range(n)) / (n * (n * n - 1))
            check(rho >= 0.8, "COE's yield follows tonnage",
                  f"Spearman rho {rho:.2f} over {n} porphyries (tonnage Mt, COE amount): "
                  + ", ".join(f"{t / 1e6:.1f}/{am}" for t, am, _ in sorted(samples)))
        else:
            check(False, "COE's yield follows tonnage", f"only {len(samples)} sampled porphyries had a vein in a probed chunk (need 3)")
    if not cdg:
        for t in ("CDG reports oil over a trap", "CDG reports no oil elsewhere"):
            check(False, t, "no `probe cdg` answers (second boot did not run)")
        return
    over = [p for p in cdg if p.get("trap")]
    amounts = [(p["chunk"], (p.get("cdg") or {}).get("amount"), p["trap"]["charge_mb"]) for p in over]
    check(bool(over) and all(p.get("agrees") is not False for p in over) and any((am or 0) > 0 for _, am, _ in amounts),
          "CDG reports oil over a trap",
          f"{len(over)} probed chunks over {len(plan['traps'])} trap site(s); (chunk, CDG amount mB, trap charge mB) {amounts[:4]}; CDG installed: {cdg[0].get('cdg_installed')}")
    empties = {(chunk_of(e["x"]) + dx, chunk_of(e["z"]) + dz) for e in plan["empty"] for dx in range(-RING, RING + 1) for dz in range(-RING, RING + 1)}
    none = [p for p in cdg if tuple(p["chunk"]) in empties and not p.get("trap")]
    nonzero = [(p["chunk"], p.get("deep_time_amount"), (p.get("cdg") or {}).get("amount")) for p in none
               if (p.get("deep_time_amount") or 0) != 0 or ((p.get("cdg") or {}).get("amount") or 0) != 0]
    check(bool(none) and not nonzero, "CDG reports no oil elsewhere",
          f"{len(none)} chunks with no trap, nonzero: {len(nonzero)} {nonzero[:3]}")


# ---------------------------------------------------------------------------------------------
# scale comparison (item 7): presets, the region, per-scale statistics
# ---------------------------------------------------------------------------------------------
SCALES = (("1:100", 100.0), ("1:300", 300.0), ("earth", 1221.0))
REGION = 2048   # blocks: the same angular window at every scale (C is the same in blocks)


def read_npy(path):
    """A float32 .npy as (rows, cols, channels, flat array) with no numpy."""
    import array
    with open(path, "rb") as f:
        assert f.read(6) == b"\x93NUMPY"
        major, _ = f.read(2)
        hl = int.from_bytes(f.read(2 if major == 1 else 4), "little")
        head = f.read(hl).decode()
        shape = tuple(int(v) for v in re.search(r"'shape': \(([^)]*)\)", head).group(1).replace(" ", "").strip(",").split(","))
        data = array.array("f")
        data.frombytes(f.read())
    if sys.byteorder == "big":
        data.byteswap()
    rows, cols = shape[0], shape[1]
    ch = shape[2] if len(shape) > 2 else 1
    return rows, cols, ch, data


def heights(world_data, box, step, out):
    r = run([DEEPTIME, "heightmap", "--world-data", world_data, f"--box={box[0]},{box[1]},{box[2]},{box[3]}", "--step", str(step), "--out", out])
    if r.returncode:
        sys.exit("heightmap failed: " + (r.stderr or r.stdout)[-400:])
    rows, cols, ch, data = read_npy(out)
    return rows, cols, [data[i * ch] for i in range(rows * cols)]


def pct(vals, q):
    s = sorted(vals)
    return s[min(len(s) - 1, int(q * (len(s) - 1)))]


def cmd_scale_preset(a):
    """One preset per scale: the built-in earthlike_quick_32k with scale.m_per_block_h changed."""
    base = json.load(open(os.path.join(REPO, "crates", "deeptime-core", "presets", "earthlike_quick_32k.json")))
    os.makedirs(a.work, exist_ok=True)
    for label, m in SCALES:
        d = json.loads(json.dumps(base))
        d["scale"]["m_per_block_h"] = m
        d["name"] = f"earthlike_quick_32k_h{int(m)}"
        d["description"] = f"{base.get('description', '')} Phase 1 exit scale comparison: {m:g} m per block ({label})."
        path = os.path.join(a.work, f"preset-h{int(m)}.json")
        json.dump(d, open(path, "w"), indent=1)
        print(path, "radius_km", round(d["scale"]["circumference_blocks"] * m / 1000 / (2 * math.pi), 1))


def cmd_scale_region(a):
    """The window every scale is rendered over: land, high relief, off the poles and the seam, chosen on
    the 1:300 bake (all three share the plate history, so it is the same patch of planet)."""
    wd = os.path.join(a.work, "h300", "planet", "deeptime")
    m = json.load(open(os.path.join(wd, "manifest.json")))
    c = m["scale"]["circumference_blocks"]
    sea = m["scale"].get("sea_level_y", 63)
    step = 32
    zp = c // 4
    box = (-c // 2, -zp, c // 2, zp)
    rows, cols, h = heights(wd, box, step, os.path.join(a.work, "h300-coarse.npy"))
    win = REGION // step
    best = None
    for r0 in range(int(rows * 0.2), rows - win - int(rows * 0.2), win // 4):
        for c0 in range(win, cols - 2 * win, win // 4):   # off the seam
            v = [h[r * cols + cc] for r in range(r0, r0 + win) for cc in range(c0, c0 + win)]
            land = sum(1 for x in v if x > sea) / len(v)
            if land < 0.8:
                continue
            score = pct(v, 0.95) - pct(v, 0.5)
            if best is None or score > best[0]:
                best = (score, r0, c0, land, pct(v, 0.95))
    if best is None:
        sys.exit("scale-region: no 2048-block window on land")
    _, r0, c0, land, p95 = best
    x0 = box[0] + c0 * step
    z0 = box[1] + r0 * step
    region = {"x0": x0, "z0": z0, "x1": x0 + REGION, "z1": z0 + REGION, "cx": x0 + REGION // 2, "cz": z0 + REGION // 2,
              "land": round(land, 3), "p95_h": round(p95, 1), "circumference": c}
    json.dump(region, open(os.path.join(a.work, "region.json"), "w"))
    print(json.dumps(region))


def cmd_scale_stats(a):
    region = json.load(open(os.path.join(a.work, "region.json")))
    out = []
    for label, m in SCALES:
        wd = os.path.join(a.work, f"h{int(m)}", "planet", "deeptime")
        if not os.path.exists(os.path.join(wd, "manifest.json")):
            continue
        mf = json.load(open(os.path.join(wd, "manifest.json")))
        sea = mf["scale"].get("sea_level_y", 63)
        step = 4
        rows, cols, h = heights(wd, (region["x0"], region["z0"], region["x1"], region["z1"]), step, os.path.join(a.work, f"h{int(m)}-region.npy"))
        land = [x for x in h if x > sea]
        # Slope: rise per block of run between neighbouring samples (4 blocks apart), over land.
        steep = tot = 0
        sl = []
        for r in range(rows - 1):
            for cc in range(cols - 1):
                x = h[r * cols + cc]
                if x <= sea:
                    continue
                g = max(abs(h[r * cols + cc + 1] - x), abs(h[(r + 1) * cols + cc] - x)) / step
                sl.append(g)
                tot += 1
                steep += g >= 1.0
        sl.sort()
        out.append({"label": label, "m_per_block_h": m, "m_per_block_v": mf["scale"].get("m_per_block_v"), "circumference_blocks": mf["scale"]["circumference_blocks"],
                    "radius_km": round(mf["scale"]["circumference_blocks"] * m / 1000 / (2 * math.pi)),
                    "region_km": round(REGION * m / 1000, 1),
                    "land_share": round(len(land) / len(h), 3), "max_y": round(max(h)), "p95_y": round(pct(h, 0.95)), "median_land_y": round(pct(land, 0.5)) if land else None,
                    "median_slope": round(sl[len(sl) // 2], 3) if sl else None, "p95_slope": round(sl[int(0.95 * (len(sl) - 1))], 3) if sl else None,
                    "steep_share": round(steep / max(tot, 1), 3)})
        print(json.dumps(out[-1]))
    json.dump(out, open(os.path.join(a.work, "scale-stats.json"), "w"), indent=1)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("pick")
    p.add_argument("--world", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--radius", type=int, default=384, help="Chunky radius around the porphyry (blocks): BlueMap shows the range around it")
    p.set_defaults(f=cmd_pick)
    p = sub.add_parser("serve")
    p.add_argument("--profile", default="mic")
    p.add_argument("--plan", required=True)
    p.add_argument("--timeout", type=int, default=3000)
    p.set_defaults(f=cmd_serve)
    p = sub.add_parser("geology")
    p.add_argument("--world", required=True)
    p.add_argument("--plan", required=True)
    p.add_argument("--rcon", required=True)
    p.add_argument("--out", required=True)
    p.set_defaults(f=cmd_geology)
    p = sub.add_parser("compat")
    p.add_argument("--plan", required=True)
    p.add_argument("--rcon", required=True)
    p.set_defaults(f=cmd_compat)
    for name, fn in (("scale-preset", cmd_scale_preset), ("scale-region", cmd_scale_region), ("scale-stats", cmd_scale_stats)):
        p = sub.add_parser(name)
        p.add_argument("--work", required=True)
        p.set_defaults(f=fn)
    a = ap.parse_args()
    a.f(a)


if __name__ == "__main__":
    main()
