import copy,pathlib,sys,json,unittest
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]/"scripts/ci"))
from typespec_refresh_guard import check_refresh,current_status
ROOT=pathlib.Path(__file__).resolve().parents[1]
P=json.loads((ROOT/"contracts/schema-parity/package.json").read_text())
R=json.loads((ROOT/"contracts/schema-parity/package-lock.review.json").read_text())
class RefreshAdmissionTests(unittest.TestCase):
 def test_current_affected_lock_is_not_security_certified(self):
  self.assertEqual(current_status()["promotion"],"blocked")
 def test_synthetic_refreshed_peer_roots_require_matching_review(self):
  p=copy.deepcopy(P);r=copy.deepcopy(R)
  for key in ("@typespec/compiler","@typespec/json-schema"):
   p["devDependencies"][key]="1.16.0";r["reviewed_roots"][key]="1.16.0"
  self.assertTrue(check_refresh(p,r))
  r["reviewed_roots"]["@typespec/compiler"]="1.15.0"
  with self.assertRaises(ValueError):check_refresh(p,r)
 def test_digest_and_node_version_are_mandatory(self):
  p=copy.deepcopy(P);r=copy.deepcopy(R)
  for key in ("@typespec/compiler","@typespec/json-schema"):
   p["devDependencies"][key]="1.16.0";r["reviewed_roots"][key]="1.16.0"
  r["package_lock_sha256"]="unset"
  with self.assertRaises(ValueError):check_refresh(p,r)
if __name__=="__main__":unittest.main()
