from __future__ import annotations

from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR_PATH = ROOT / "scripts" / "ci" / "validate_source_authority.py"
MANIFEST_PATH = ROOT / "contracts" / "source-authority" / "authority.v1.json"

SPEC = importlib.util.spec_from_file_location("validate_source_authority", VALIDATOR_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("unable to load source-authority validator")
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)

MANIFEST = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


class SourceAuthorityContractTests(unittest.TestCase):
    def test_reviewed_contract_is_valid(self) -> None:
        self.assertEqual(VALIDATOR.validate_manifest(deepcopy(MANIFEST)), [])

    def test_typespec_lane_cannot_consume_json_lane_source(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["source_lanes"]["typespec"]["source_inputs"].append(
            "json-schema-source"
        )
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.source_lanes.typespec.source_inputs:must-be-typespec-only",
            errors,
        )

    def test_json_lane_cannot_substitute_typespec_emitted_schema(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["source_lanes"]["json-schema-openapi"]["source_inputs"] = [
            "typespec-emitted-json-schema"
        ]
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.source_lanes.json-schema-openapi.source_inputs:"
            "forbidden-production-input:typespec-emitted-json-schema",
            errors,
        )
        self.assertIn(
            "$.source_lanes.json-schema-openapi.source_inputs:"
            "must-be-independent-json-schema-openapi-only",
            errors,
        )

    def test_both_lanes_must_emit_sql(self) -> None:
        for lane in ("typespec", "json-schema-openapi"):
            with self.subTest(lane=lane):
                manifest = deepcopy(MANIFEST)
                manifest["source_lanes"][lane]["required_outputs"].remove("sql")
                errors = VALIDATOR.validate_manifest(manifest)
                self.assertIn(
                    f"$.source_lanes.{lane}.required_outputs:missing:sql",
                    errors,
                )

    def test_typespec_lane_must_retain_protobuf_and_grpc_outputs(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["source_lanes"]["typespec"]["required_outputs"].remove(
            "protobuf-descriptors"
        )
        manifest["source_lanes"]["typespec"]["required_outputs"].remove(
            "grpc-services"
        )
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.source_lanes.typespec.required_outputs:missing:protobuf-descriptors",
            errors,
        )
        self.assertIn(
            "$.source_lanes.typespec.required_outputs:missing:grpc-services",
            errors,
        )

    def test_orm_generation_must_be_independent(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["comparisons"]["diesel-seaorm"]["generation_policy"] = (
            "diesel-generates-seaorm"
        )
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.comparisons.diesel-seaorm.generation_policy:"
            "must-be-independent-from-same-pinned-catalog",
            errors,
        )

    def test_discrepancy_must_stop_all_promotion_lanes(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["discrepancy_policy"]["status"] = "WARN_ONLY"
        manifest["discrepancy_policy"]["blocks"].remove("deployment")
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.discrepancy_policy.status:must-equal-STOPPED_FOR_EVALUATION",
            errors,
        )
        self.assertIn(
            "$.discrepancy_policy.blocks:missing:deployment",
            errors,
        )

    def test_derived_translation_cannot_feed_a_production_lane(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["derived_translations"]["may_feed_production_lane"] = True
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.derived_translations.may_feed_production_lane:must-be-false",
            errors,
        )

    def test_database_evidence_lanes_cannot_be_collapsed(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["convergence"]["database_lanes"] = ["postgresql"]
        errors = VALIDATOR.validate_manifest(manifest)
        self.assertIn(
            "$.convergence.database_lanes:"
            "must-separate-postgresql-and-cockroachdb",
            errors,
        )

    def test_cli_returns_stopped_status_for_invalid_contract(self) -> None:
        manifest = deepcopy(MANIFEST)
        manifest["source_lanes"]["typespec"]["required_outputs"].remove("sql")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "invalid.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            completed = subprocess.run(
                [sys.executable, str(VALIDATOR_PATH), "--manifest", str(path)],
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
        self.assertEqual(completed.returncode, 2)
        result = json.loads(completed.stderr)
        self.assertEqual(result["status"], "STOPPED_FOR_EVALUATION")
        self.assertIn(
            "$.source_lanes.typespec.required_outputs:missing:sql",
            result["errors"],
        )


if __name__ == "__main__":
    unittest.main()
