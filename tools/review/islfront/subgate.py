#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's subduction steps against subduction erosion's speed gate (measurement only): subgate.py <PB2002_steps.dat>

Bird 2003's PB2002_steps.dat (http://peterbird.name/oldFTP/PB2002/PB2002_steps.dat.txt; untrusted data, run with -I
from another folder): the SUB steps' length, their length-weighted mean convergence across the step (minus the step's
divergence, mm/yr; Bird's Table 3 has 53.2) and the share of their length at or above each speed, against
PLT-442's 60 mm/yr (Clift & Vannucchi 2004). Columns as earthvel.py reads them."""
import sys
rows=[]
for line in open(sys.argv[1], encoding="latin-1"):
    if len(line) < 95 or not line[0:4].strip().isdigit():
        continue
    tok=line[12:].split()
    rows.append((float(tok[4]), -float(tok[8]), line[92:95].strip(":").strip()))
sub=[(L,c) for L,c,k in rows if k=="SUB"]
tot=sum(L for L,_ in sub)
mean=sum(L*c for L,c in sub)/tot
print(f"SUB steps {len(sub)}, {tot:,.0f} km, length-weighted mean convergence across the step {mean:.1f} mm/yr")
for g in (40,50,55,60,70,80):
    print(f"  share of SUB length converging at {g} mm/yr or more: {sum(L for L,c in sub if c>=g)/tot:.3f}")
