#!/usr/bin/env python3
"""Bring a newer PC (EffectCraft) version into this phone project without losing the phone parts.

    python mobile-sync/sync.py <new_pc_dir> <mobile_dir> [--out <dir>] [--dry-run]

`new_pc_dir`  the unzipped new PC version (the folder with Cargo.toml in it).
`mobile_dir`  this phone project (the folder that holds mobile-sync/).
`--out`       where the result goes (default: <mobile_dir>-synced). The phone project is not touched.

How it works. `pc-base.json` holds the SHA-256 of every file of the PC version this project was
last synced to, and `pc-base/` holds that version's copy of the few PC files the phone project
edited (the "hook files"). For every file of the new PC version:
  * PC did not change it                      -> the phone project's file stays
  * PC changed it, the phone did not edit it  -> the new PC file is copied
  * PC changed it, the phone edited it        -> three-way merge (base = pc-base, mine = phone,
                                                 theirs = new PC) with `git merge-file`; a conflict
                                                 is left in the file as <<<<<<< markers and listed
  * new PC file                               -> added
  * file the PC deleted                       -> deleted when the phone never edited it
Files in MOBILE_WINS keep the phone project's version; Cargo.lock comes from the new PC version
(cargo adds the phone-only crates again on the next build).
Afterwards pc-base.json and pc-base/ are refreshed to the new PC version in the result.
See MOBILE_SYNC_GUIDE.md for what to run after this.
"""
import hashlib, json, os, shutil, subprocess, sys, tempfile

MOBILE_WINS = {"README.md", ".gitignore", ".cargo/config.toml"}
TAKE_PC = {"Cargo.lock"}
SKIP_DIRS = {"target", ".git"}


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def walk(root):
    out = {}
    for d, dirs, files in os.walk(root):
        dirs[:] = [x for x in dirs if x not in SKIP_DIRS]
        for name in files:
            p = os.path.join(d, name)
            out[os.path.relpath(p, root).replace(os.sep, "/")] = p
    return out


def pc_version(pc_dir):
    try:
        for line in open(os.path.join(pc_dir, "Cargo.toml"), encoding="utf-8"):
            if line.startswith("version") and '"' in line:
                return line.split('"')[1]
    except OSError:
        pass
    return "?"


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dry = "--dry-run" in sys.argv
    out_dir = None
    if "--out" in sys.argv:
        out_dir = sys.argv[sys.argv.index("--out") + 1]
        args = [a for a in args if a != out_dir]
    if len(args) != 2:
        print(__doc__)
        sys.exit(2)
    new_pc, mobile = (os.path.abspath(a) for a in args)
    out_dir = os.path.abspath(out_dir or mobile.rstrip("/\\") + "-synced")
    if not os.path.isfile(os.path.join(new_pc, "Cargo.toml")):
        sys.exit(f"{new_pc} is not a PC project folder (no Cargo.toml)")
    base_json = os.path.join(mobile, "mobile-sync", "pc-base.json")
    base = json.load(open(base_json, encoding="utf-8"))
    base_hash, base_files_dir = base["files"], os.path.join(mobile, "mobile-sync", "pc-base")

    pc, mb = walk(new_pc), walk(mobile)
    changed, merged, conflicts, added, removed, needs_manual, took = [], [], [], [], [], [], []
    plan = {}  # rel -> ("copy", src) | ("merge", text, conflict) | ("delete",)

    for rel, src in sorted(pc.items()):
        if rel.startswith("mobile-sync/"):
            continue
        new_h = sha(src)
        if rel not in base_hash:
            if rel in mb and sha(mb[rel]) != new_h:
                needs_manual.append(f"{rel}: new in PC but the phone project has its own file of that name")
            else:
                plan[rel] = ("copy", src)
                added.append(rel)
            continue
        if base_hash[rel] == new_h:
            continue  # PC unchanged
        changed.append(rel)
        if rel in MOBILE_WINS:
            continue
        if rel in TAKE_PC:
            plan[rel] = ("copy", src)
            took.append(rel)
            continue
        mine = mb.get(rel)
        if mine is None:
            plan[rel] = ("copy", src)  # the phone project had dropped it: PC's comes back
            added.append(rel)
            continue
        if sha(mine) == base_hash[rel]:
            plan[rel] = ("copy", src)  # the phone never edited it
            continue
        base_copy = os.path.join(base_files_dir, rel)
        if not os.path.isfile(base_copy):
            needs_manual.append(f"{rel}: edited by the phone project and changed by PC, but pc-base/ has no copy of the old PC file")
            continue
        with tempfile.TemporaryDirectory() as tmp:
            m = os.path.join(tmp, "mine")
            shutil.copyfile(mine, m)
            r = subprocess.run(["git", "merge-file", "-p", "-L", "phone", "-L", "pc-old", "-L", "pc-new", m, base_copy, src], capture_output=True)
            if r.returncode < 0 or (r.returncode > 0 and not r.stdout):
                needs_manual.append(f"{rel}: git merge-file failed: {r.stderr.decode(errors='replace').strip()}")
                continue
            plan[rel] = ("merge", r.stdout, r.returncode > 0)
            (conflicts if r.returncode > 0 else merged).append(rel)

    for rel in sorted(base_hash):
        if rel not in pc and rel in mb and not rel.startswith("mobile-sync/"):
            if sha(mb[rel]) == base_hash[rel]:
                plan[rel] = ("delete",)
                removed.append(rel)
            else:
                needs_manual.append(f"{rel}: deleted by PC but edited by the phone project")

    print(f"PC {base.get('pc_version', '?')} -> {pc_version(new_pc)}: {len(changed)} PC files changed")
    for title, items in (("copied from PC (phone never edited them)", [r for r, p in plan.items() if p[0] == 'copy' and r not in added and r not in took]),
                         ("merged cleanly (phone edits kept)", merged), ("added", added), ("deleted", removed), ("taken from PC", took)):
        if items:
            print(f"\n{title}: {len(items)}")
            for r in items[:60]:
                print("   ", r)
            if len(items) > 60:
                print(f"    ... and {len(items) - 60} more")
    if conflicts:
        print(f"\nCONFLICTS ({len(conflicts)}): fix the <<<<<<< / >>>>>>> blocks, keeping the phone hook and PC's new code:")
        for r in conflicts:
            print("   ", r)
    if needs_manual:
        print(f"\nNEEDS A LOOK ({len(needs_manual)}):")
        for r in needs_manual:
            print("   ", r)
    if dry:
        print("\n--dry-run: nothing written")
        return

    if os.path.exists(out_dir):
        sys.exit(f"{out_dir} already exists; remove it or pass --out")
    shutil.copytree(mobile, out_dir, ignore=shutil.ignore_patterns(*SKIP_DIRS))
    for rel, step in plan.items():
        dst = os.path.join(out_dir, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if step[0] == "copy":
            shutil.copyfile(step[1], dst)
        elif step[0] == "merge":
            with open(dst, "wb") as f:
                f.write(step[1])
        elif step[0] == "delete" and os.path.exists(dst):
            os.remove(dst)

    # The next sync starts from this PC version.
    hooks = [r for r in base.get("hook_files", []) if r in pc]
    new_base = {"pc_version": pc_version(new_pc), "hook_files": base.get("hook_files", []), "files": {r: sha(p) for r, p in pc.items() if not r.startswith("mobile-sync/")}}
    sync_out = os.path.join(out_dir, "mobile-sync")
    json.dump(new_base, open(os.path.join(sync_out, "pc-base.json"), "w", encoding="utf-8"), indent=0, sort_keys=True)
    shutil.rmtree(os.path.join(sync_out, "pc-base"), ignore_errors=True)
    for rel in hooks:
        dst = os.path.join(sync_out, "pc-base", rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copyfile(pc[rel], dst)
    print(f"\nWritten to {out_dir}\nNext: follow 'After a sync' in mobile-sync/MOBILE_SYNC_GUIDE.md.")
    sys.exit(1 if conflicts or needs_manual else 0)


if __name__ == "__main__":
    main()
