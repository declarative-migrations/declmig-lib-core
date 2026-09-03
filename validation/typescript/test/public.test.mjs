import assert from "node:assert/strict";
import test from "node:test";
import { parsePublic, safeParsePublic } from "../dist/public.js";

const problem = { type: "urn:test", title: "Invalid request", status: 400, requestId: "req-1" };

test("preserves exact request metadata and accepts bounds", () => {
  const exact = { requestId: " req-1 ", traceId: " trace-1 ", locale: "en" };
  assert.deepEqual(parsePublic("request-meta", exact), exact);
  assert.equal(safeParsePublic("request-meta", { requestId: "r".repeat(128), traceId: "t".repeat(128), locale: "l".repeat(64) }).success, true);
  assert.deepEqual(parsePublic("page-query", { limit: 1 }), { limit: 1 });
  assert.deepEqual(parsePublic("page-query", { limit: 100, cursor: "c".repeat(512) }), { limit: 100, cursor: "c".repeat(512) });
  assert.equal(safeParsePublic("problem-details", { ...problem, status: 599, detail: "d".repeat(4096) }).success, true);
});

for (const [schema, name, value] of [
  ["request-meta", "missing trace", { requestId: "req-1" }],
  ["request-meta", "empty request", { requestId: "", traceId: "trace-1" }],
  ["request-meta", "oversized request", { requestId: "r".repeat(129), traceId: "trace-1" }],
  ["request-meta", "short locale", { requestId: "req-1", traceId: "trace-1", locale: "e" }],
  ["request-meta", "client identity", { requestId: "req-1", traceId: "trace-1", userId: "client-supplied" }],
  ["page-query", "missing limit", {}],
  ["page-query", "zero limit", { limit: 0 }],
  ["page-query", "oversized limit", { limit: 101 }],
  ["page-query", "fractional limit", { limit: 1.5 }],
  ["page-query", "empty cursor", { limit: 50, cursor: "" }],
  ["page-query", "unknown offset", { limit: 50, offset: 1 }],
  ["problem-details", "low status", { ...problem, status: 399 }],
  ["problem-details", "high status", { ...problem, status: 600 }],
  ["problem-details", "fractional status", { ...problem, status: 400.5 }],
  ["problem-details", "empty title", { ...problem, title: "" }],
  ["problem-details", "oversized detail", { ...problem, detail: "d".repeat(4097) }],
  ["problem-details", "internal code", { ...problem, internalCode: "secret" }],
]) test(`rejects ${schema}: ${name}`, () => assert.equal(safeParsePublic(schema, value).success, false));
