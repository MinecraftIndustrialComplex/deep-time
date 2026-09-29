#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Fill in crates/deeptime-core/tests/share-codes.json (P1-13, plan §5.1).

Each case holds canonical JSON plus the share code each side writes for it:

  rust  `deeptime preset encode <file>` (run through cargo; needs the dev shell)
  java  neoforge/build/share-codes-java.json, which the mod's ShareCodeTest writes on every run

Both test suites decode both codes, so a change of either encoder that the other cannot read
fails a test. Run from the repo root after `./gradlew test` (on the Mac: tools/remote/mac.sh).
"""

import json
import os
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FIXTURE = os.path.join(REPO, "crates", "deeptime-core", "tests", "share-codes.json")
JAVA = os.path.join(REPO, "neoforge", "build", "share-codes-java.json")


def main():
    fixture = json.load(open(FIXTURE))
    java = json.load(open(JAVA)) if os.path.exists(JAVA) else {}
    if not java:
        print(f"share_code_fixture: {JAVA} is missing; java codes are left as they are", file=sys.stderr)
    for case in fixture["cases"]:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            f.write(case["json"])
            path = f.name
        try:
            out = subprocess.run(["cargo", "run", "-q", "-p", "deeptime-cli", "--", "preset", "encode", path],
                                 cwd=REPO, check=True, capture_output=True, text=True).stdout.strip()
        finally:
            os.remove(path)
        case["rust"] = out
        if case["name"] in java:
            case["java"] = java[case["name"]]
        print(f"{case['name']}: rust {len(case['rust'])} chars, java {len(case['java'])} chars")
    with open(FIXTURE, "w") as f:
        json.dump(fixture, f, indent=2, ensure_ascii=False)
        f.write("\n")


if __name__ == "__main__":
    main()
