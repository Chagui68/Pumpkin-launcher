# Per-plugin status from a PatchBukkit server log: python analyze.py <log> <plugins_dir>
import re, sys, zipfile, os, collections

log = open(sys.argv[1], encoding="utf-8", errors="replace").read()
log = re.sub(r"\x1b\[[0-9;]*m", "", log)
lines = log.splitlines()

names = {}
for f in sorted(os.listdir(sys.argv[2])):
    if not f.endswith(".jar"):
        continue
    try:
        z = zipfile.ZipFile(os.path.join(sys.argv[2], f))
        y = next((z.read(n).decode("utf-8", "replace") for n in ("paper-plugin.yml", "plugin.yml") if n in z.namelist()), "")
        m = re.search(r"^name:\s*['\"]?([^'\"\n]+)", y, re.M)
        if m:
            names[m.group(1).strip()] = f
    except Exception as e:
        print("bad jar", f, e)

status = {}
for n in names:
    en = any(f"Enabling {n} v" in l for l in lines)
    err = [i for i, l in enumerate(lines) if f"while enabling {n} v" in l or f"Could not load '{n}" in l or (f"[{n}]" in l and " ERROR " in l)]
    first = ""
    for i in err:
        for l in lines[i:i + 3]:
            if re.search(r"(Exception|Error):", l):
                first = l.strip()[:150]
                break
        if first:
            break
    fail_enable = any(f"while enabling {n} v" in l for l in lines)
    disabled_early = any(f"Disabling {n} v" in l for l in lines[: next((i for i, l in enumerate(lines) if "Stopping the server" in l), len(lines))])
    if not en:
        s = "NOT LOADED"
    elif fail_enable or disabled_early:
        s = "FAILED ENABLE"
    elif err:
        s = "ENABLED, ERRORS"
    else:
        s = "OK"
    status[n] = (s, len(err), first)

c = collections.Counter(v[0] for v in status.values())
print(dict(c))
for n, (s, k, first) in sorted(status.items(), key=lambda kv: kv[1][0]):
    if s != "OK":
        print(f"{s:16} {n:28} errors={k} {first}")
print("OK:", ", ".join(n for n, v in sorted(status.items()) if v[0] == "OK"))
for pat in ("Unknown dependency", "UnknownDependencyException", "Could not load", "Ambiguous plugin name", "does not seem to be compatible"):
    for l in lines:
        if pat in l:
            print("NOTE", l.strip()[:170])
print("stopped:", "The server has stopped" in log, "| pbtest:", (re.findall(r"Total: \d+ +Passed: \d+ +Failed: \d+", log) or ["-"])[-1])
