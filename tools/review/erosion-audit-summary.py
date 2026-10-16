#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Compact tables from `deeptime erosion-audit --json` files (tools/review/erosion-audit.sh).

    python3 tools/review/erosion-audit-summary.py <name>=<audit.json> ...
"""
import json
import sys


def main():
    for arg in sys.argv[1:]:
        name, path = arg.split("=", 1)
        r = json.load(open(path))
        print("=====", name)
        h = r.get("history")
        if h:
            hy = h["hypsometry"]
            le = hy["land_elevation_m"]
            a = hy["land_share_above_m"]
            print(f"land {hy['land_share']}  land elevation p50 {le['p50']} p90 {le['p90']} "
                  f"p99 {le['p99']}  above 1/2/3 km {a['1000']} {a['2000']} {a['3000']}")
            print("rates, m/Myr: region  area-time share | erosion mean p50 p90 p99 | "
                  "deposition mean p50 p90 | erosion share >100 >500")
            for k, v in h["rates_by_region"].items():
                e, d = v["erosion_m_myr"], v["deposition_m_myr"]
                print(f"  {k:10s} {e['area_share']:6} | {e['mean']:7} {e['p50']:7} {e['p90']:7} "
                      f"{e['p99']:7} | {d['mean']:7} {d['p50']:7} {d['p90']:7} | "
                      f"{v['erosion_share_above_100_m_myr']} {v['erosion_share_above_500_m_myr']}")
            bands = h.get("land_erosion_by_elevation_m", {})
            if bands:
                print("land erosion by elevation band, m/Myr: band  area-time share | mean p50 p90")
                for k, v in bands.items():
                    print(f"  {k:10s} {v['area_share']:6} | {v['mean']:7} {v['p50']:7} {v['p90']:7}")
            print("parcel totals, km: region  eroded mean p50 p90 | deposited mean p50 p90 | "
                  "eroded last 500 Myr mean | share eroded >10 km | peak frame erosion p50 p90 m/Myr")
            for k, v in h["parcel_totals_by_final_region"].items():
                e, d = v["eroded_km"], v["deposited_km"]
                e5, p = v["eroded_last_500_myr_km"], v["peak_frame_erosion_m_myr"]
                print(f"  {k:10s} {e['mean']:6} {e['p50']:6} {e['p90']:6} | {d['mean']:6} "
                      f"{d['p50']:6} {d['p90']:6} | {e5['mean']:6} | {v['share_eroded_over_10_km']} | "
                      f"{p['p50']} {p['p90']}")
        p = r.get("porphyries")
        if p:
            print("porphyries", p["porphyries"], p["fates"], "eroded km p10/50/90",
                  p["eroded_km_p10_p50_p90"], "at exhumation", p["exhumation_eroded_km_p10_p50_p90"])
            for row in p["rows"]:
                if row["site"] >= 0:
                    print("  site", row["site"], "formed", row["formed_ma"], "Ma", row["fate"],
                          "eroded", row["eroded_m"], "buried", row["buried_m"], "most net erosion",
                          row.get("exhumed_max_m"), "ledger centre",
                          row["ledger_centre_depth_m"], "m; centre",
                          row["centre_depth_blocks"], "blocks under the ground")
            dep = sorted(r2["centre_depth_blocks"] for r2 in p["rows"]
                         if r2["site"] >= 0 and r2["centre_depth_blocks"] is not None)
            if dep:
                q = lambda f: dep[round((len(dep) - 1) * f)]
                print(f"  sites {len(dep)}: centre depth under the ground, blocks p50 {q(0.5)} "
                      f"p90 {q(0.9)} max {dep[-1]}; deeper than 100 blocks: "
                      f"{sum(1 for x in dep if x > 100)}")
            bur = sorted(r2["buried_m"] for r2 in p["rows"] if r2["buried_m"] is not None)
            if bur:
                q = lambda f: bur[round((len(bur) - 1) * f)]
                print("  buried_m over all porphyries p10/50/90:", q(0.1), q(0.5), q(0.9))


if __name__ == "__main__":
    main()
