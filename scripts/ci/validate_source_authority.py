#!/usr/bin/env python3
"""Validate the dual-source schema authority contract without external packages."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence

SCHEMA_VERSION = 1
POLICY_ID = "declmig.dual-source-authority.v1"
SOURCE_LANES = {"typespec", "json-schema-openapi"}

REQUIRED_OUTPUTS = {
    "typespec": {
        "semantic-ir",
        "sql",
        "protobuf-descriptors",
        "grpc-services",
        "wire-client-types",
        "wire-clients",
    },
    "json-schema-openapi": {
        "semantic-ir",
        "sql",
        "client-interfaces",
        "client-types",
        "write-clients",
    },
}

REQUIRED_COMPARISONS = {
    "sql-catalog": {
        "typespec-sql",
        "json-schema-openapi-sql",
    },
    "shared-client-semantics": {
        "typespec-client-semantic-manifest",
        "json-schema-openapi-client-semantic-manifest",
    },
    "diesel-seaorm": {
        "diesel-projection",
        "seaorm-projection",
    },
}

REQUIRED_CONVERGENCE_PARTICIPANTS = {
    "typespec-sql-catalog",
    "json-schema-openapi-sql-catalog",
    "reviewed-dpm-desired-catalog",
    "diesel-projection",
    "seaorm-projection",
    "shadow-live-catalog-readback",
}

REQUIRED_BLOCKS = {
    "migration-promotion",
    "orm-promotion",
    "package-publication",
    "client-publication",
    "merge",
    "deployment",
}

REQUIRED_EVIDENCE = {
    "source-revisions",
    "compiler-generator-versions",
    "input-output-catalog-digests",
    "minimal-semantic-diff",
    "exact-head-ci",
}


def _object(value: Any, path: str, errors: list[str]) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        errors.append(f"{path}:expected-object")
        return {}
    return value


def _string_set(value: Any, path: str, errors: list[str]) -> set[str]:
    if not isinstance(value, list):
        errors.append(f"{path}:expected-array")
        return set()

    result: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item != item.strip():
            errors.append(f"{path}[{index}]:expected-trimmed-string")
            continue
        if item in result:
            errors.append(f"{path}:duplicate:{item}")
        result.add(item)
    return result


def _require_subset(
    required: Iterable[str],
    actual: set[str],
    path: str,
    errors: list[str],
) -> None:
    for item in sorted(set(required) - actual):
        errors.append(f"{path}:missing:{item}")


def _require_exact_keys(
    value: Mapping[str, Any],
    expected: set[str],
    path: str,
    errors: list[str],
) -> None:
    for key in sorted(expected - set(value)):
        errors.append(f"{path}:missing-key:{key}")
    for key in sorted(set(value) - expected):
        errors.append(f"{path}:unknown-key:{key}")


def validate_manifest(value: Any) -> list[str]:
    """Return stable, public-safe error codes. An empty list means valid."""

    errors: list[str] = []
    root = _object(value, "$", errors)
    _require_exact_keys(
        root,
        {
            "schema_version",
            "policy_id",
            "derived_translations",
            "source_lanes",
            "comparisons",
            "convergence",
            "discrepancy_policy",
        },
        "$",
        errors,
    )

    if root.get("schema_version") != SCHEMA_VERSION:
        errors.append("$.schema_version:must-equal-1")
    if root.get("policy_id") != POLICY_ID:
        errors.append("$.policy_id:unexpected")

    derived = _object(root.get("derived_translations"), "$.derived_translations", errors)
    if derived.get("comparison_only") is not True:
        errors.append("$.derived_translations.comparison_only:must-be-true")
    if derived.get("may_feed_production_lane") is not False:
        errors.append("$.derived_translations.may_feed_production_lane:must-be-false")
    provenance = _string_set(
        derived.get("required_provenance"),
        "$.derived_translations.required_provenance",
        errors,
    )
    _require_subset(
        {
            "origin_lane",
            "source_revision",
            "compiler_name",
            "compiler_version",
            "input_sha256",
            "output_sha256",
        },
        provenance,
        "$.derived_translations.required_provenance",
        errors,
    )

    lanes = _object(root.get("source_lanes"), "$.source_lanes", errors)
    _require_exact_keys(lanes, SOURCE_LANES, "$.source_lanes", errors)

    lane_inputs: dict[str, set[str]] = {}
    for lane_id in sorted(SOURCE_LANES):
        lane = _object(lanes.get(lane_id), f"$.source_lanes.{lane_id}", errors)
        _require_exact_keys(
            lane,
            {
                "authority",
                "source_inputs",
                "forbidden_production_inputs",
                "required_outputs",
            },
            f"$.source_lanes.{lane_id}",
            errors,
        )
        if lane.get("authority") != "independent-top-level":
            errors.append(f"$.source_lanes.{lane_id}.authority:must-be-independent-top-level")
        source_inputs = _string_set(
            lane.get("source_inputs"),
            f"$.source_lanes.{lane_id}.source_inputs",
            errors,
        )
        forbidden = _string_set(
            lane.get("forbidden_production_inputs"),
            f"$.source_lanes.{lane_id}.forbidden_production_inputs",
            errors,
        )
        outputs = _string_set(
            lane.get("required_outputs"),
            f"$.source_lanes.{lane_id}.required_outputs",
            errors,
        )
        lane_inputs[lane_id] = source_inputs
        _require_subset(
            REQUIRED_OUTPUTS[lane_id],
            outputs,
            f"$.source_lanes.{lane_id}.required_outputs",
            errors,
        )
        forbidden_overlap = source_inputs & forbidden
        for item in sorted(forbidden_overlap):
            errors.append(
                f"$.source_lanes.{lane_id}.source_inputs:forbidden-production-input:{item}"
            )

    if lane_inputs.get("typespec") != {"typespec-source"}:
        errors.append("$.source_lanes.typespec.source_inputs:must-be-typespec-only")
    if lane_inputs.get("json-schema-openapi") != {
        "json-schema-source",
        "openapi-source",
    }:
        errors.append(
            "$.source_lanes.json-schema-openapi.source_inputs:"
            "must-be-independent-json-schema-openapi-only"
        )

    comparisons = _object(root.get("comparisons"), "$.comparisons", errors)
    _require_exact_keys(
        comparisons,
        set(REQUIRED_COMPARISONS),
        "$.comparisons",
        errors,
    )
    for comparison_id, required_inputs in sorted(REQUIRED_COMPARISONS.items()):
        comparison = _object(
            comparisons.get(comparison_id),
            f"$.comparisons.{comparison_id}",
            errors,
        )
        inputs = _string_set(
            comparison.get("required_inputs"),
            f"$.comparisons.{comparison_id}.required_inputs",
            errors,
        )
        _require_subset(
            required_inputs,
            inputs,
            f"$.comparisons.{comparison_id}.required_inputs",
            errors,
        )

    sql_comparison = _object(
        comparisons.get("sql-catalog"),
        "$.comparisons.sql-catalog",
        errors,
    )
    if sql_comparison.get("materialization") != "separate-disposable-databases":
        errors.append(
            "$.comparisons.sql-catalog.materialization:"
            "must-use-separate-disposable-databases"
        )
    if sql_comparison.get("primary_oracle") != "normalized-catalog-equivalence":
        errors.append(
            "$.comparisons.sql-catalog.primary_oracle:"
            "must-use-normalized-catalog-equivalence"
        )
    sql_dimensions = _string_set(
        sql_comparison.get("required_dimensions"),
        "$.comparisons.sql-catalog.required_dimensions",
        errors,
    )
    _require_subset(
        {
            "native-types",
            "nullability",
            "defaults",
            "generated-expressions",
            "primary-keys",
            "unique-keys",
            "foreign-keys",
            "checks",
            "indexes",
            "vector-dimensions",
            "rls",
            "policies",
            "grants",
            "ownership",
        },
        sql_dimensions,
        "$.comparisons.sql-catalog.required_dimensions",
        errors,
    )

    client_comparison = _object(
        comparisons.get("shared-client-semantics"),
        "$.comparisons.shared-client-semantics",
        errors,
    )
    if client_comparison.get("mapping_policy") != "explicit-reviewed-versioned":
        errors.append(
            "$.comparisons.shared-client-semantics.mapping_policy:"
            "must-be-explicit-reviewed-versioned"
        )
    client_dimensions = _string_set(
        client_comparison.get("required_dimensions"),
        "$.comparisons.shared-client-semantics.required_dimensions",
        errors,
    )
    _require_subset(
        {
            "field-identity",
            "wire-names",
            "required-optional-nullable",
            "scalar-widths",
            "formats",
            "unions",
            "discriminators",
            "enums",
            "constraints",
            "operations",
            "request-response-errors",
        },
        client_dimensions,
        "$.comparisons.shared-client-semantics.required_dimensions",
        errors,
    )

    orm_comparison = _object(
        comparisons.get("diesel-seaorm"),
        "$.comparisons.diesel-seaorm",
        errors,
    )
    if (
        orm_comparison.get("generation_policy")
        != "independent-from-same-pinned-catalog"
    ):
        errors.append(
            "$.comparisons.diesel-seaorm.generation_policy:"
            "must-be-independent-from-same-pinned-catalog"
        )
    orm_dimensions = _string_set(
        orm_comparison.get("required_dimensions"),
        "$.comparisons.diesel-seaorm.required_dimensions",
        errors,
    )
    _require_subset(
        {
            "table-identity",
            "ordered-keys",
            "columns",
            "native-types",
            "nullability",
            "defaults",
            "relations",
            "shared-behavior-fixtures",
        },
        orm_dimensions,
        "$.comparisons.diesel-seaorm.required_dimensions",
        errors,
    )

    convergence = _object(root.get("convergence"), "$.convergence", errors)
    if convergence.get("all_required") is not True:
        errors.append("$.convergence.all_required:must-be-true")
    participants = _string_set(
        convergence.get("participants"),
        "$.convergence.participants",
        errors,
    )
    _require_subset(
        REQUIRED_CONVERGENCE_PARTICIPANTS,
        participants,
        "$.convergence.participants",
        errors,
    )
    database_lanes = _string_set(
        convergence.get("database_lanes"),
        "$.convergence.database_lanes",
        errors,
    )
    if database_lanes != {"postgresql", "cockroachdb"}:
        errors.append(
            "$.convergence.database_lanes:"
            "must-separate-postgresql-and-cockroachdb"
        )

    discrepancy = _object(
        root.get("discrepancy_policy"),
        "$.discrepancy_policy",
        errors,
    )
    if discrepancy.get("status") != "STOPPED_FOR_EVALUATION":
        errors.append(
            "$.discrepancy_policy.status:must-equal-STOPPED_FOR_EVALUATION"
        )
    if discrepancy.get("deduplicate_by") != "deterministic-fingerprint":
        errors.append(
            "$.discrepancy_policy.deduplicate_by:"
            "must-use-deterministic-fingerprint"
        )
    if discrepancy.get("silent_winner_forbidden") is not True:
        errors.append(
            "$.discrepancy_policy.silent_winner_forbidden:must-be-true"
        )
    if (
        discrepancy.get("resume_policy")
        != "human-reviewed-repair-or-expiring-waiver-then-clean-rerun"
    ):
        errors.append("$.discrepancy_policy.resume_policy:unexpected")
    blocks = _string_set(
        discrepancy.get("blocks"),
        "$.discrepancy_policy.blocks",
        errors,
    )
    _require_subset(
        REQUIRED_BLOCKS,
        blocks,
        "$.discrepancy_policy.blocks",
        errors,
    )
    evidence = _string_set(
        discrepancy.get("required_evidence"),
        "$.discrepancy_policy.required_evidence",
        errors,
    )
    _require_subset(
        REQUIRED_EVIDENCE,
        evidence,
        "$.discrepancy_policy.required_evidence",
        errors,
    )

    return sorted(set(errors))


def load_manifest(path: Path) -> Any:
    if path.is_symlink() or not path.is_file():
        raise ValueError("manifest-path-invalid")
    if path.stat().st_size > 1024 * 1024:
        raise ValueError("manifest-too-large")
    return json.loads(path.read_text(encoding="utf-8"))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("contracts/source-authority/authority.v1.json"),
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        value = load_manifest(args.manifest)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "status": "STOPPED_FOR_EVALUATION",
                    "errors": [f"manifest-load:{type(exc).__name__}"],
                },
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        return 64

    errors = validate_manifest(value)
    result = {
        "schema_version": 1,
        "policy_id": POLICY_ID,
        "status": "VALID" if not errors else "STOPPED_FOR_EVALUATION",
        "errors": errors,
    }
    stream = sys.stdout if not errors else sys.stderr
    print(json.dumps(result, sort_keys=True), file=stream)
    return 0 if not errors else 2


if __name__ == "__main__":
    raise SystemExit(main())
