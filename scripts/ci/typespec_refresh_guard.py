"""Fail-closed advisories/readiness of reviewed TypeSpec peer lock.

The current reviewed TypeSpec 1.15.0 graph remains blocked until npm 10.9.2
regenerates both uncompressed/compressed locks and the complete review receipt.
No one may mark the existing 1.15.0 evidence as security-certified.
"""
import json,pathlib,re
ROOT=pathlib.Path(__file__).resolve().parents[1]
def check_refresh(candidate,review):
    pkg=candidate["devDependencies"]
    roots=review["reviewed_roots"]
    required=("@typespec/compiler","@typespec/json-schema")
    if any(pkg.get(n)!=roots.get(n) for n in required):
        raise ValueError("root/lock review dependency drift")
    versions=[pkg[n] for n in required]
    if any(not re.fullmatch(r"1\.(?:1[6-9]|[2-9][0-9])\.[0-9]+",v) for v in versions):
        raise ValueError("TypeSpec security floor not met: review 1.16.x+ compiler and emitter")
    for field in ("package_inventory_sha256","package_lock_sha256","package_lock_gzip_sha256"):
        if not re.fullmatch("[0-9a-f]{64}",review.get(field,"")):
            raise ValueError("missing immutable lock digest")
    if review.get("node_version")!="22.16.0" or review.get("npm_version")!="10.9.2":
        raise ValueError("unreviewed lock generation toolchain")
    return True
def current_status():
    p=json.loads((ROOT/"contracts/schema-parity/package.json").read_text())
    r=json.loads((ROOT/"contracts/schema-parity/package-lock.review.json").read_text())
    try:
        check_refresh(p,r)
    except ValueError as e:
        return {"promotion":"blocked","reason":str(e)}
    return {"promotion":"candidate","reason":"still requires npm audit and TJSV"}
if __name__=="__main__":print(json.dumps(current_status(),sort_keys=True))
