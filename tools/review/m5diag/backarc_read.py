"""Pool backarc.py's runs: backarc_read.py <ba.jsonl> [<ba.jsonl> ...] (means over runs; one block per file)."""
import json
import sys

import numpy as np


def load(path):
    return [json.loads(x) for x in open(path) if x.strip()]


def get(r, path):
    for k in path.split("."):
        if r is None:
            return None
        r = r.get(k) if isinstance(r, dict) else None
    return r


def m(rows, path):
    v = [get(r, path) for r in rows]
    v = [x for x in v if x is not None]
    return float(np.mean(v)) if v else float("nan")


def block(path):
    R = load(path)
    print(f"==== {path}: {len(R)} runs")
    print(f"K model_km {m(R, 'kin.K_model_km'):.0f}  spread {m(R, 'kin.K_spread_p90_p10'):.4f}"
          f"  trench paired {m(R, 'trench_paired'):.3f}  map align {m(R, 'map_align'):.3f}")
    print(f"SUB km last {m(R, 'sub_km_last'):.0f}  contacts {m(R, 'kin.contacts'):.1f}"
          f"  >=500 km {m(R, 'kin.contacts_ge500km'):.1f}  upper plates {m(R, 'kin.upper_plates'):.1f}"
          f"  lower {m(R, 'kin.lower_plates'):.1f}  top5 share {m(R, 'kin.contact_km_top5_share'):.2f}")
    print("contact km " + " ".join(f"{q} {m(R, 'kin.contact_km.' + q):.0f}" for q in ("p25", "p50", "p75", "p90"))
          + f" max {m(R, 'kin.contact_km_max'):.0f}")
    print(f"pairs by arc cells {m(R, 'pair_by_arc'):.3f}  trench cell's own plate is u {m(R, 'own_is_u'):.3f}")
    for kk, groups in (("kin", ("all", "oo", "oc", "coll")), ("kin_conv", ("all", "oo", "oc"))):
        print(f"-- {kk} (mm/yr; Vup > 0 toward the trench)  sub cells {m(R, kk + '.sub_cells'):.0f}"
              f"  contacts {m(R, kk + '.contacts'):.1f} >=500 km {m(R, kk + '.contacts_ge500km'):.1f}"
              f"  upper plates {m(R, kk + '.upper_plates'):.1f}")
        for g in groups:
            s = f"{g:5s} share {m(R, f'{kk}.{g}.share'):.3f} "
            for v in ("vup", "vsub", "vc", "vpar"):
                s += f" {v} " + "/".join(f"{m(R, f'{kk}.{g}.{v}.{q}'):.1f}" for q in ("p10", "p50", "p90"))
            s += f"  vc<0 {m(R, f'{kk}.{g}.vc_neg'):.3f}  vup>vsub {m(R, f'{kk}.{g}.vup_gt_vsub'):.3f}"
            print(s)
            print("      vup bands " + " ".join(f"{b} {m(R, f'{kk}.{g}.vup_bands.{b}'):.3f}"
                                             for b in ("ret_fast", "ret", "neutral", "adv", "adv_fast"))
                  + f"  vup<0 {m(R, f'{kk}.{g}.vup_neg'):.3f}")
            print("      NNR vup " + "/".join(f"{m(R, f'{kk}.{g}.vup_nnr.{q}'):.1f}" for q in ("p10", "p25", "p50", "p75", "p90"))
                  + " vsub " + "/".join(f"{m(R, f'{kk}.{g}.vsub_nnr.{q}'):.1f}" for q in ("p10", "p50", "p90"))
                  + "  bands " + " ".join(f"{b} {m(R, f'{kk}.{g}.vup_nnr_bands.{b}'):.3f}"
                                          for b in ("ret_fast", "ret", "neutral", "adv", "adv_fast"))
                  + f"  vup<0 {m(R, f'{kk}.{g}.vup_nnr_neg'):.3f}")
        print(f"      net rotation at the equator {m(R, kk + '.net_rot_mm_yr'):.1f} mm/yr")
    print("-- back-arc regime (4)")
    ba = "grid.backarc."
    print(f"share planet {m(R, ba + 'share_planet'):.4f}  oceanic {m(R, ba + 'oceanic'):.3f}"
          f"  area/SUB km {m(R, ba + 'area_over_sub_km'):.0f}  own-plate SUB front <=800 {m(R, ba + 'own_plate_front_le800'):.3f}"
          f"  land {m(R, ba + 'land'):.3f} (cont {m(R, ba + 'cont_land'):.3f}, oce {m(R, ba + 'oce_land'):.3f})  hi {m(R, ba + 'hi'):.3f}")
    print("d km " + " ".join(f"{q} {m(R, ba + 'd_km.' + q):.0f}" for q in ("p10", "p25", "p50", "p75", "p90")))
    print("by d " + " ".join(f"{b} {m(R, ba + 'by_d.' + b):.3f}" for b in ("d0_200", "d200_440", "d440_800", "d800p")))
    print("by vup " + " ".join(f"{b} {m(R, ba + 'by_vup.' + b):.3f}" for b in ("ret_fast", "ret", "neutral", "adv", "adv_fast")))
    print("-- behind SUB fronts (<=800 km, own plate) by Vup band")
    for b in ("ret_fast", "ret", "neutral", "adv", "adv_fast"):
        p = f"grid.behind.{b}."
        print(f"{b:9s} planet {m(R, p + 'cells_share_planet'):.4f} cont {m(R, p + 'cont'):.3f} land {m(R, p + 'land'):.3f}"
              f" hi {m(R, p + 'hi'):.3f} T_cont p50 {m(R, p + 'T_cont_p50_km'):.1f} km  hi/land {m(R, p + 'hi_of_land_L'):.4f}")
    print(f"-- high ground (share of land), M4 {m(R, 'grid.M4'):.4f}")
    cols = ("m4", "ret_fast", "ret", "neutral", "adv", "adv_fast", "coll", "none",
            "d0_200", "d200_440", "d440_800", "d800p", "dcoll_le800", "beh_oo", "beh_oc")
    print(f"{'class':17s}" + "".join(f"{c:>10s}" for c in cols))
    for c in ("coll_young", "coll_10_50", "coll_50_150", "coll_150p", "andean_young", "andean_old",
              "arc_untagged", "plateau_untagged", "oceanic", "other"):
        print(f"{c:17s}" + "".join(f"{m(R, f'grid.hi_by_class.{c}.{x}'):10.4f}" for x in cols))
    print("land by context " + " ".join(f"{c} {m(R, 'grid.land_ctx.' + c):.3f}" for c in
                                        ("ret_fast", "ret", "neutral", "adv", "adv_fast", "coll", "none"))
          + f"  behind oo {m(R, 'grid.land_beh_oo'):.3f} oc {m(R, 'grid.land_beh_oc'):.3f}")
    print("-- record, steps after -900 Myr (mean km)")
    names = ("oo", "oc", "coll", "ridge", "rift", "other")
    mix = np.mean([r["rec"]["mix_km_mean"] for r in R], 0)
    print("mix " + " ".join(f"{n} {x:.0f}" for n, x in zip(names, mix)) + f"  total {mix.sum():.0f}")
    mixl = np.mean([r["rec"]["mix_km_last"] for r in R], 0)
    print("mix last " + " ".join(f"{n} {x:.0f}" for n, x in zip(names, mixl)))
    for key, edges in (("slab", (0, 50, 100, 150, 300, 600)), ("normal", (0, 5, 10, 20, 40, 80)),
                       ("age", (0, 6, 12, 24, 50, 100))):
        arr = np.mean([r["rec"][f"{key}_mean"] for r in R], 0)  # 2 x 6
        tot = arr.sum()
        print(f"{key:6s} " + " ".join(f">={e}: {arr[:, i].sum() / tot:.3f}" for i, e in enumerate(edges))
              + f"  (oo {arr[0].sum():.0f} km, oc {arr[1].sum():.0f} km)")
        for ci, cn in ((0, "oo"), (1, "oc")):
            print(f"   {cn} " + " ".join(f">={e}: {arr[ci, i] / max(arr[ci].sum(), 1e-9):.3f}" for i, e in enumerate(edges)))
    for key, edges in (("oo_over_cont", (0, 0.01, 0.05, 0.15, 0.3, 0.5)), ("oo_under_cont", (0, 0.01, 0.05, 0.15, 0.3, 0.5)),
                       ("oo_coast", (0, 150, 300, 600, 1200, 2400)), ("oo_pair", ("initiation", "rift", "other"))):
        if all(f"{key}_mean" in r["rec"] for r in R):
            arr = np.mean([r["rec"][f"{key}_mean"] for r in R], 0)
            print(f"{key:13s} " + " ".join(f"{e}: {x:.0f}" for e, x in zip(edges, arr)) + f"  (sum {arr.sum():.0f})")
    inc = np.mean([r["rec"]["incipient_mean"] for r in R], 0)
    ben = np.mean([r["rec"]["benioff_mean"] for r in R], 0)
    print(f"incipient oo {inc[0]:.0f} oc {inc[1]:.0f}  benioff oo {ben[0]:.0f} oc {ben[1]:.0f}"
          f"  other converging {m(R, 'rec.other_converging_mean'):.0f}"
          f"  plates mean {m(R, 'rec.plates_mean'):.1f} last {m(R, 'rec.plates_last'):.1f}")


for f in sys.argv[1:]:
    block(f)
