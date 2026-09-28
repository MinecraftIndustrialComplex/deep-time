#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Summarise Gradle's JUnit XML reports so no acceptance depends on console output.

Usage: junit-summary.py <dir with TEST-*.xml>
Prints: tests=<n> failures=<f> errors=<e> skipped=<s>; exits 1 on any failure or error,
or when the directory holds no reports.
"""
import glob
import os
import sys
import xml.etree.ElementTree as ET

d = sys.argv[1] if len(sys.argv) > 1 else "neoforge/build/test-results/test"
files = sorted(glob.glob(os.path.join(d, "TEST-*.xml")))
totals = {"tests": 0, "failures": 0, "errors": 0, "skipped": 0}
for f in files:
    root = ET.parse(f).getroot()
    for k in totals:
        totals[k] += int(root.get(k, "0"))
    for case in root.iter("testcase"):
        for bad in ("failure", "error"):
            if case.find(bad) is not None:
                print(f"{bad.upper()}: {case.get('classname')}.{case.get('name')}")
print(" ".join(f"{k}={v}" for k, v in totals.items()))
sys.exit(1 if not files or totals["failures"] or totals["errors"] else 0)
