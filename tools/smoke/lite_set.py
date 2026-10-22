#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The lite mod set: only the mods Deep Time's generation uses (Ben, 2026-10-22).

    lite_set.py derive --mods <mods dir> --side client|server [--json F] [--write-keep F] [--check-keep F] [--quiet]
    lite_set.py apply  <set dir> --side client|server [--excluded F --excluded-why F] [--prune-files]
    lite_set.py survey --mods <mods dir>

Why (Ben: "strip down the pack when I'm doing a deeptime test to just the blocks that are used for the
deeptime generation"): the full MIC pack (about 158 mods on the server, 181 on the client) makes a
Deep Time test client laggy. The lite set is derived, never hand-listed, so it stays right as Deep
Time's data changes. A jar is kept when one of these names it, and every keep carries its reason:

  1. named     Deep Time's generation tables name something of its namespace: every JSON string that is
               a resource id (or #tag) of a namespace other than minecraft, deeptime, c and neoforge in
               Deep Time's data (neoforge/src/main/resources/data, crates/deeptime-core/data) and in the
               MIC rock-tag datapack (Docs/datapacks/deeptime-mic-rock-tags, which the play server and
               every -PwithMic run enable). The namespace maps to the jar that provides it: a jar whose
               mod id is the namespace, else the jar that defines it (blockstates, biomes, loot tables
               under that namespace in its assets and data).
  2. tag-member  a material c:<name> is the block tag #c:ores/<name> (OreBlockResolver), and Deep Time's
               data also names tags such as #c:is_beach: the members of those tags that the set's jars
               add (data/<ns>/tags/<kind>/**), followed through nested tags, name their jars too.
  3. jar-data  a jar that ships Deep Time data of its own (data/deeptime/** or data/<ns>/deeptime/**), or
               a pack extension of its tables.
  4. compat    Deep Time gates generation on a mod: the packages under mixin/compat/ of the mod
               (createoreexcavation, createdieselgenerators, ...) are named by mod id.
  5. router    a jar that overrides vanilla's overworld router or biome list (data/minecraft/worldgen/
               noise_settings, density_function, noise, multi_noise_biome_source_parameter_list):
               ClimateMappingBuilder samples the pack's own minecraft:overworld router at a world's first
               start, so a pack that rewrites it makes a different climate mapping.
  6. core      profiles/mic-lite.core: Deep Time, Toroidal World, Distant Horizons, the dev map, the
               client performance mods, and any mod the router rule cannot see (Tectonic works by mixin).
  7. requires  the required dependencies (neoforge.mods.toml type "required", or mandatory = true) of
               every kept jar, closed transitively; the client set counts side BOTH and CLIENT, the
               server set BOTH and SERVER.

Everything else is left out, even when it adds its own overworld features. `derive` prints the table
(and the namespaces that are named but that no jar of this set provides); `apply` moves the jars that
are not kept to <set dir>/mods-excluded (as tools/smoke/exclude_mods.py does) and records them; with
--prune-files it also drops from a client stage the pack files that only the left-out mods use (see
PRUNE below) and rewrites the stage's packwiz.json to match. `survey` lists every jar's mod ids,
namespaces and Deep Time hooks (a development aid for this rule set).

Nothing here is heavy but it reads every jar: run it on the test VM (tools/remote/mac.sh), not the laptop.
"""
import argparse
import json
import os
import re
import shutil
import sys
import tomllib
import zipfile
from collections import OrderedDict, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
PROFILES = os.path.join(HERE, "profiles")
CORE_FILE = os.path.join(PROFILES, "mic-lite.core")

# Namespaces whose content every set has (or that are Deep Time's own): never a reason to keep a jar.
SKIP_NS = {"minecraft", "deeptime", "c", "neoforge", "forge", "fabric", "kubejs"}
# Prefixes of strings that look like ids but are not (DOIs, URLs).
JUNK_NS = {"doi", "http", "https", "ftp", "mailto", "isbn", "arxiv", "url"}
# JSON keys whose subtree is prose or citations.
SKIP_KEYS = {"sources", "source", "notes", "note", "comment", "comments", "description", "citation",
             "citations", "ledger", "reference", "references"}
RESOURCE = re.compile(r"^(#?)([a-z0-9_.-]+):([a-z0-9_./-]+)$")
# Mod ids that are the platform, not jars.
PLATFORM = {"minecraft", "neoforge", "forge", "fml", "javafml", "lowcodefml", "java"}
TAG_KINDS = ("block", "worldgen/biome", "fluid", "item", "entity_type")
ROUTER_DIRS = ("noise_settings", "density_function", "noise", "multi_noise_biome_source_parameter_list")


# ---------------------------------------------------------------------------
# Jars
# ---------------------------------------------------------------------------
class Jar:
    """One mod jar: its mod ids (nested jars included), dependencies, namespaces and hooks."""

    def __init__(self, path):
        self.path = path
        self.name = os.path.basename(path)
        self.mod_ids = []          # own and jar-in-jar mod ids
        self.own_ids = []
        self.deps = []             # (owner id, dep id, side) of required dependencies (own mods only)
        self.score = defaultdict(int)       # namespace -> how much of it this jar defines
        self.ns = set()                     # every namespace under data/ and assets/
        self.blockstates = defaultdict(set)  # namespace -> block paths with an assets blockstate
        self.biomes = defaultdict(set)       # namespace -> worldgen/biome paths
        self.tags = {}                       # (kind, ns, path) -> zip entry name
        self.deeptime_data = []              # entries of Deep Time's data this jar ships
        self.router = []                     # vanilla router / biome list overrides
        self.error = None
        try:
            self._read()
        except zipfile.BadZipFile:
            self.error = "not a zip"

    def _toml(self, z):
        for name in ("META-INF/neoforge.mods.toml", "META-INF/mods.toml"):
            if name in z.namelist():
                try:
                    return tomllib.loads(z.read(name).decode("utf-8", "replace"))
                except tomllib.TOMLDecodeError as e:
                    print(f"   lite_set: cannot parse {name} of {self.name}: {e}", file=sys.stderr)
        return None

    def _read(self):
        with zipfile.ZipFile(self.path) as z:
            names = z.namelist()
            data = self._toml(z)
            if data:
                for m in data.get("mods", []):
                    if "modId" in m:
                        self.own_ids.append(m["modId"])
                for owner, lst in (data.get("dependencies") or {}).items():
                    if isinstance(lst, dict):
                        lst = [lst]
                    for d in lst:
                        if "type" in d:
                            required = str(d["type"]).lower() == "required"
                        else:
                            required = bool(d.get("mandatory", True))
                        if required and "modId" in d:
                            self.deps.append((owner, d["modId"], str(d.get("side", "BOTH")).upper()))
            self.mod_ids = list(self.own_ids)
            for n in names:
                if n.startswith("META-INF/jarjar/") and n.endswith(".jar"):
                    try:
                        import io
                        with zipfile.ZipFile(io.BytesIO(z.read(n))) as nz:
                            nd = self._toml(nz)
                            for m in (nd or {}).get("mods", []):
                                if "modId" in m and m["modId"] not in self.mod_ids:
                                    self.mod_ids.append(m["modId"])
                    except (zipfile.BadZipFile, KeyError):
                        pass
            for n in names:
                parts = n.split("/")
                if len(parts) < 3 or parts[0] not in ("data", "assets"):
                    continue
                ns = parts[1]
                self.ns.add(ns)
                rest = parts[2:]
                if parts[0] == "assets" and rest[0] == "blockstates" and n.endswith(".json"):
                    self.score[ns] += 1
                    self.blockstates[ns].add("/".join(rest[1:])[:-5])
                elif parts[0] == "data":
                    if rest[:2] == ["worldgen", "biome"] and n.endswith(".json"):
                        self.score[ns] += 1
                        self.biomes[ns].add("/".join(rest[2:])[:-5])
                    elif rest[0] in ("loot_table", "loot_tables") and len(rest) > 2 and rest[1] == "blocks":
                        self.score[ns] += 1
                    elif rest[0] == "tags" and n.endswith(".json"):
                        tail = "/".join(rest[1:])
                        for kind in sorted(TAG_KINDS, key=len, reverse=True):
                            if tail.startswith(kind + "/"):
                                self.tags[(kind, ns, tail[len(kind) + 1:-5])] = n
                                break
                    if ns == "minecraft" and rest[0] == "worldgen" and len(rest) > 2 and rest[1] in ROUTER_DIRS:
                        self.router.append(n)
                    if (ns == "deeptime" and not n.startswith("data/deeptime/lang")) or (len(rest) > 1 and rest[0] == "deeptime"):
                        self.deeptime_data.append(n)

    def read_json(self, entry):
        with zipfile.ZipFile(self.path) as z:
            return json.loads(z.read(entry).decode("utf-8", "replace"))


class Index:
    """Every jar of a mods folder, and lookups by mod id and namespace."""

    def __init__(self, mods_dir):
        self.jars = OrderedDict()
        for f in sorted(os.listdir(mods_dir)):
            if f.endswith(".jar"):
                j = Jar(os.path.join(mods_dir, f))
                if j.error:
                    print(f"   lite_set: {f}: {j.error}", file=sys.stderr)
                self.jars[f] = j
        self.own_by_id = defaultdict(list)   # jars whose own mods.toml declares the id
        self.by_id = defaultdict(list)       # ... plus the jars that carry it as a jar-in-jar
        for j in self.jars.values():
            for i in j.mod_ids:
                self.by_id[i].append(j.name)
            for i in j.own_ids:
                self.own_by_id[i].append(j.name)

    def jars_for_id(self, mod_id, kept=()):
        """Jars to keep so that mod_id is loaded: every jar that declares it, else (a library several
        mods nest) one nesting jar, none when a kept jar already carries it."""
        if mod_id in self.own_by_id:
            return list(self.own_by_id[mod_id])
        nested = self.by_id.get(mod_id, [])
        if any(n in kept for n in nested):
            return []
        return nested[:1]

    def provider(self, ns):
        """(jar names, how) of the jars that provide a namespace: mod id first, else the best definer."""
        if ns in self.by_id:
            return self.jars_for_id(ns), f"mod id {ns}"
        best, top = [], 0
        for j in self.jars.values():
            s = j.score.get(ns, 0)
            if s > top:
                best, top = [j.name], s
            elif s == top and s > 0:
                best.append(j.name)
        if best:
            return best, f"defines {ns}:* ({top} blockstates, biomes and block loot tables)"
        return [], ""

    def tag_members(self, kind, ns, path, seen=None):
        """Leaf ids (ns:path strings) of a tag across every jar, nested tags followed."""
        seen = seen if seen is not None else set()
        key = (kind, ns, path)
        if key in seen:
            return []
        seen.add(key)
        out = []
        for j in self.jars.values():
            entry = j.tags.get(key)
            if not entry:
                continue
            try:
                values = j.read_json(entry).get("values", [])
            except (ValueError, KeyError):
                continue
            for v in values:
                v = v.get("id") if isinstance(v, dict) else v
                if not isinstance(v, str):
                    continue
                m = RESOURCE.match(v)
                if not m:
                    continue
                if m.group(1):
                    out.extend(self.tag_members(kind, m.group(2), m.group(3), seen))
                else:
                    out.append((v, j.name))
        return out


# ---------------------------------------------------------------------------
# Deep Time's named ids
# ---------------------------------------------------------------------------
def walk(obj, key=None):
    """Yields (string, parent key) for every string value and every id-shaped key outside prose."""
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k.startswith("_") or k in SKIP_KEYS:
                continue
            if RESOURCE.match(k):
                yield k, "(key)"
            yield from walk(v, k)
    elif isinstance(obj, list):
        for v in obj:
            yield from walk(v, key)
    elif isinstance(obj, str):
        yield obj, key


def scan_root(root, label):
    """{(is_tag, ns, path, material?): [relative file, ...]} for the ids named in a data folder."""
    refs = defaultdict(list)
    for dirpath, _, files in os.walk(root):
        for f in sorted(files):
            if not f.endswith(".json"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, f), root)
            try:
                with open(os.path.join(dirpath, f), encoding="utf-8") as fh:
                    obj = json.load(fh)
            except (ValueError, OSError):
                continue
            for s, key in walk(obj):
                m = RESOURCE.match(s)
                if not m or "//" in s or m.group(2) in JUNK_NS:
                    continue
                refs[(bool(m.group(1)), m.group(2), m.group(3), key == "material")].append(f"{label}: {rel}")
    return refs


def default_roots():
    roots = [("Deep Time data", os.path.join(REPO, "neoforge/src/main/resources/data")),
             ("Deep Time core data", os.path.join(REPO, "crates/deeptime-core/data"))]
    tags = os.environ.get("DEEPTIME_ROCK_TAGS") or os.path.join(REPO, "../../Docs/datapacks/deeptime-mic-rock-tags")
    tags = os.path.normpath(tags)
    if os.path.isdir(tags):
        roots.append(("MIC rock tags", tags))
    else:
        print(f"   lite_set: WARNING no MIC rock-tag datapack at {tags} (DEEPTIME_ROCK_TAGS; mac.sh --pack syncs it): "
              "the mods its blocks name are not counted", file=sys.stderr)
    return roots


def compat_ids():
    """Mod ids of Deep Time's mixin/compat/<mod> packages (generation hooks that apply only with that mod)."""
    base = os.path.join(REPO, "neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/mixin/compat")
    return sorted(d for d in os.listdir(base) if os.path.isdir(os.path.join(base, d))) if os.path.isdir(base) else []


def read_core(path=CORE_FILE):
    core = OrderedDict()
    if os.path.isfile(path):
        for line in open(path, encoding="utf-8"):
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            mod, _, why = line.partition(" ")
            core[mod] = why.strip() or "core"
    return core


# ---------------------------------------------------------------------------
# Derivation
# ---------------------------------------------------------------------------
def dep_applies(side_of_dep, set_side):
    return side_of_dep in ("BOTH", set_side.upper())


def derive(mods_dir, side, roots=None, core_path=CORE_FILE):
    ix = Index(mods_dir)
    why = defaultdict(list)       # jar name -> [(category, text)]
    absent = defaultdict(list)    # namespace -> files that name it, no jar provides it
    warnings = []

    def keep(jar, cat, text):
        if (cat, text) not in why[jar]:
            why[jar].append((cat, text))

    # 1 and 2: names and tag members.
    refs = defaultdict(list)
    for label, root in (roots if roots is not None else default_roots()):
        for k, files in scan_root(root, label).items():
            refs[k].extend(files)
    tag_requests = defaultdict(list)   # (ns, path) -> files
    by_ns = defaultdict(list)          # ns -> [(display id, file)]
    for (is_tag, ns, path, material), files in sorted(refs.items()):
        if ns == "c" and not is_tag and "/" not in path:
            tag_requests[("c", f"ores/{path}")].extend(files)     # a material: #c:ores/<name>
            continue
        if is_tag and ns not in ("minecraft", "deeptime"):
            tag_requests[(ns, path)].extend(files)
        if ns in SKIP_NS:
            continue
        by_ns[ns].extend((("#" if is_tag else "") + f"{ns}:{path}", f) for f in files)
    for ns, items in sorted(by_ns.items()):
        jars, how = ix.provider(ns)
        if not jars:
            absent[ns] = sorted({f for _, f in items})
            continue
        for jar in jars:
            first_id, first_file = items[0]
            more = len({i for i, _ in items}) - 1
            keep(jar, "named", f"{first_id} in {first_file}" + (f" (+{more} more names; {how})" if more else f" ({how})"))
    for (ns, path), files in sorted(tag_requests.items()):
        members = []
        for kind in TAG_KINDS:
            members.extend(ix.tag_members(kind, ns, path))
        by_jar = defaultdict(set)
        for leaf, tagjar in members:
            lns = leaf.split(":", 1)[0]
            if lns in SKIP_NS:
                continue
            for jar in ix.provider(lns)[0]:
                by_jar[jar].add(leaf)
        for jar, leaves in by_jar.items():
            keep(jar, "tag-member", f"{sorted(leaves)[0]}" + (f" (+{len(leaves) - 1})" if len(leaves) > 1 else "") +
                 f" in #{ns}:{path}, named by {files[0]}")

    # 3: Deep Time data inside a jar.
    for j in ix.jars.values():
        if j.deeptime_data:
            keep(j.name, "jar-data", f"ships Deep Time data ({j.deeptime_data[0]}" + (f", +{len(j.deeptime_data) - 1}" if len(j.deeptime_data) > 1 else "") + ")")
    # 4: compat mixin packages.
    for mod in compat_ids():
        for jar in ix.jars_for_id(mod):
            keep(jar, "compat", f"Deep Time gates generation on {mod} (mixin/compat/{mod})")
    # 5: router overrides.
    for j in ix.jars.values():
        if j.router:
            keep(j.name, "router", f"overrides the overworld router or biome list ({j.router[0]}" + (f", +{len(j.router) - 1}" if len(j.router) > 1 else "") + ")")
    # 6: the core.
    core = read_core(core_path)
    for mod, text in core.items():
        if mod not in ix.by_id:
            warnings.append(f"core mod {mod} is not in this set (fine: only those present are kept)")
            continue
        for jar in ix.jars_for_id(mod):
            keep(jar, "core", text)

    # 7: required dependencies, closed.
    kept = set(why)
    queue = list(kept)
    while queue:
        name = queue.pop()
        j = ix.jars[name]
        for owner, dep, dside in j.deps:
            if dep in PLATFORM or not dep_applies(dside, side):
                continue
            if dep in j.mod_ids:        # shipped inside the same jar
                continue
            if dep not in ix.by_id:
                warnings.append(f"{name} requires {dep}, which no jar of this set provides")
                continue
            for p in ix.jars_for_id(dep, kept):
                if p not in kept:
                    kept.add(p)
                    queue.append(p)
                keep(p, "requires", f"required by {name} ({owner} -> {dep})")
    return ix, why, kept, absent, warnings


CATEGORY_ORDER = ["named", "tag-member", "jar-data", "compat", "router", "core", "requires"]


def render(ix, why, kept, absent, warnings, side):
    out = []
    drop = [n for n in ix.jars if n not in kept]
    out.append(f"== lite set ({side}): keep {len(kept)} of {len(ix.jars)} jars, leave out {len(drop)}")
    for name in ix.jars:
        if name not in kept:
            continue
        reasons = sorted(why[name], key=lambda r: CATEGORY_ORDER.index(r[0]))
        ids = ",".join(ix.jars[name].own_ids[:3]) or "-"
        out.append(f"KEEP {name}  [{ids}]")
        for cat, text in reasons[:4]:
            out.append(f"       {cat}: {text}")
        if len(reasons) > 4:
            out.append(f"       (+{len(reasons) - 4} more reasons)")
    if absent:
        out.append("== named but provided by no jar of this set (fine when the pack does not have the mod):")
        for ns, files in sorted(absent.items()):
            out.append(f"   {ns}: {files[0]}" + (f" (+{len(files) - 1} files)" if len(files) > 1 else ""))
    for w in sorted(set(warnings)):
        out.append(f"   note: {w}")
    out.append("== left out (nothing names them):")
    out.append("   " + ", ".join(drop))
    return "\n".join(out)


def keep_file_text(ix, why, kept, side):
    lines = [f"# Generated by tools/smoke/lite_set.py ({side} set): the jars kept for the Deep Time lite set and why.",
             "# Regenerate with: lite_set.py derive --mods <mods dir> --side %s --write-keep <this file>" % side,
             "# The live derivation (lite_set.py apply) is authoritative; this file is the reviewable record of one run.",
             f"# {len(kept)} kept of {len(ix.jars)}"]
    for name in ix.jars:
        if name in kept:
            reasons = sorted(why[name], key=lambda r: CATEGORY_ORDER.index(r[0]))
            lines.append(f"{name}\t{','.join(ix.jars[name].own_ids[:3]) or '-'}\t{reasons[0][0]}: {reasons[0][1]}")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# apply
# ---------------------------------------------------------------------------
# Pack files a lite client stage carries: root files and the pack's config of mods that stay. Everything
# under kubejs/, scripts/ and resourcepacks/ belongs to mods the lite set leaves out (KubeJS itself,
# the radio and weather packs), so those folders go. A config file is attributed to the mod whose id is
# its longest prefix (create-client.toml: create, createbigcannons-client.toml: createbigcannons);
# it goes when that mod is left out, and stays when its mod is kept or no mod claims it.
PRUNE_DIRS = ("kubejs/", "scripts/", "resourcepacks/")
CONFIG_DIRS = ("config/", "defaultconfigs/")


def claimed_by(rel, all_ids):
    """Mod id that owns a config path (longest id prefix at a name boundary), or None."""
    name = rel.split("/", 1)[1].lower()
    first = name.split("/", 1)[0]
    stem = re.split(r"[.]", first, maxsplit=1)[0]
    best = None
    for i in all_ids:
        low = i.lower()
        if len(low) < 3:
            continue
        for cand in (stem, first):
            if cand == low or (cand.startswith(low) and not cand[len(low)].isalnum()):
                if best is None or len(low) > len(best):
                    best = i
    return best


def prune_files(set_dir, ix, kept):
    kept_ids = {i for n in kept for i in ix.jars[n].mod_ids}
    all_ids = {i for j in ix.jars.values() for i in j.mod_ids}
    pw = os.path.join(set_dir, "packwiz.json")
    removed, recorded = [], 0
    cached = {}
    if os.path.isfile(pw):
        doc = json.load(open(pw))
        cached = doc.get("cachedFiles", {})
    else:
        doc = None
    for key in list(cached):
        loc = cached[key].get("cachedLocation") or key
        if loc.startswith("mods/"):
            continue
        drop = loc.startswith(PRUNE_DIRS)
        if loc.startswith(CONFIG_DIRS):
            owner = claimed_by(loc, all_ids)
            drop = owner is not None and owner not in kept_ids
        if drop:
            removed.append(loc)
            del cached[key]
    for loc in removed:
        p = os.path.join(set_dir, loc)
        if os.path.isfile(p):
            os.remove(p)
    for top in PRUNE_DIRS:
        shutil.rmtree(os.path.join(set_dir, top.rstrip("/")), ignore_errors=True)
    if doc is not None:
        doc["cachedFiles"] = cached
        recorded = len(cached)
        json.dump(doc, open(pw, "w"))
    return removed, recorded


def apply(set_dir, side, excluded_txt, why_txt, prune, core_path=CORE_FILE):
    mods = os.path.join(set_dir, "mods")
    out = os.path.join(set_dir, "mods-excluded")
    os.makedirs(out, exist_ok=True)
    ix, why, kept, absent, warnings = derive(mods, side, core_path=core_path)
    text = render(ix, why, kept, absent, warnings, side)
    print(text)
    drop = [n for n in ix.jars if n not in kept]
    ex = open(excluded_txt, "a") if excluded_txt else None
    wy = open(why_txt, "a") if why_txt else None
    for n in drop:
        shutil.move(os.path.join(mods, n), os.path.join(out, n))
        if ex:
            ex.write(n + "\n")
        if wy:
            wy.write(f"{n}\tlite set: no Deep Time data, tag or required dependency names it\n")
    for f in (ex, wy):
        if f:
            f.close()
    with open(os.path.join(set_dir, "lite-why.txt"), "w") as f:
        f.write(text + "\n")
    print(f"== moved {len(drop)} jar(s) to {out}; {len(kept)} stay in {mods}")
    if prune:
        removed, left = prune_files(set_dir, ix, kept)
        print(f"== pack files: {len(removed)} dropped (kubejs/, scripts/, resourcepacks/ and the config of left-out mods), {left} recorded in packwiz.json")


def survey(mods_dir):
    ix = Index(mods_dir)
    for j in ix.jars.values():
        top = sorted(j.score.items(), key=lambda kv: -kv[1])[:3]
        print(f"{j.name}\n   ids {j.own_ids} nested {[i for i in j.mod_ids if i not in j.own_ids]}"
              f"\n   defines {top}  deeptime-data {len(j.deeptime_data)}  router {len(j.router)}"
              f"\n   requires {sorted({d for _, d, s in j.deps if d not in PLATFORM})}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("derive")
    d.add_argument("--mods", required=True)
    d.add_argument("--side", choices=("client", "server"), required=True)
    d.add_argument("--json")
    d.add_argument("--write-keep", help="write the reviewable keep list to this file")
    d.add_argument("--check-keep", help="exit 3 when this keep list differs from the derivation")
    d.add_argument("--quiet", action="store_true")
    a = sub.add_parser("apply")
    a.add_argument("set_dir")
    a.add_argument("--side", choices=("client", "server"), required=True)
    a.add_argument("--excluded")
    a.add_argument("--excluded-why")
    a.add_argument("--prune-files", action="store_true")
    s = sub.add_parser("survey")
    s.add_argument("--mods", required=True)
    args = ap.parse_args()
    if args.cmd == "derive":
        ix, why, kept, absent, warnings = derive(args.mods, args.side)
        if not args.quiet:
            print(render(ix, why, kept, absent, warnings, args.side))
        if args.json:
            json.dump({"side": args.side, "keep": {n: [[c, t] for c, t in why[n]] for n in ix.jars if n in kept},
                       "drop": [n for n in ix.jars if n not in kept], "absent": absent}, open(args.json, "w"), indent=1)
        text = keep_file_text(ix, why, kept, args.side)
        if args.write_keep:
            open(args.write_keep, "w").write(text)
        if args.check_keep:
            have = [l for l in open(args.check_keep).read().splitlines() if l and not l.startswith("#")]
            want = [l for l in text.splitlines() if l and not l.startswith("#")]
            if {l.split("\t")[0] for l in have} != {l.split("\t")[0] for l in want}:
                print(f"lite_set: {args.check_keep} is out of date", file=sys.stderr)
                sys.exit(3)
    elif args.cmd == "apply":
        apply(args.set_dir, args.side, args.excluded, args.excluded_why, args.prune_files)
    else:
        survey(args.mods)


if __name__ == "__main__":
    main()
