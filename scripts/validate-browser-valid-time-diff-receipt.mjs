#!/usr/bin/env node
// Validates an A-3 browser valid-time diff receipt produced by
//   node examples/browser/bench-driver.cjs valid-time-diff <fixture> [samples]
// Every named gate is recomputed from the recorded evidence and compared with
// the gate the driver claimed, so a receipt whose verdict disagrees with its
// own numbers is rejected rather than trusted.
import fs from "node:fs";

const [receiptPath] = process.argv.slice(2);
if (!receiptPath) {
  throw new Error("usage: validate-browser-valid-time-diff-receipt.mjs <receipt>");
}
const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
const expect = (condition, message) => {
  if (!condition) throw new Error(message);
};

expect(receipt.schema === "vicia.browser-valid-time-diff.v1", "schema mismatch");
expect(
  receipt.chromeVersion === "Google Chrome for Testing 150.0.7871.115",
  "Chrome version mismatch",
);
expect(receipt.source?.trackedClean === true, "source must be clean");
expect(/^[0-9a-f]{40}$/.test(receipt.source.commit), "source commit invalid");
expect(/^[0-9a-f]{64}$/.test(receipt.source.fixtureSha256), "fixture hash invalid");
expect(/^[0-9a-f]{64}$/.test(receipt.source.wasmSha256), "WASM hash invalid");

// The request must be the native gate's request. A receipt measured with a
// different attribute, scope size, or instant pair is not comparable with the
// native baseline in docs/BENCHMARKS.md and is not admissible as A-3 evidence.
expect(receipt.request?.attribute === ":status/value", "diff attribute mismatch");
expect(receipt.request.entityCount === 128, "diff entity count mismatch");
expect(receipt.request.validAtBefore === 1_593_561_600_000, "before instant mismatch");
expect(receipt.request.validAtAfter === 1_625_097_600_000, "after instant mismatch");
expect(receipt.request.limit === 1_000, "diff limit mismatch");
expect(Number.isInteger(receipt.samples) && receipt.samples >= 20, "too few warm samples");

// The fixture must be the 1M base plus the 128 valid-time receipt entities.
expect(
  receipt.import?.result?.stats?.headerNodeCount === 1_000_256,
  "fixture fact count mismatch",
);

const entitySet = receipt.scopes?.entity_set?.result;
const attributeScope = receipt.scopes?.attribute_scope?.result;
expect(entitySet && attributeScope, "both diff scopes must be present");
const exactRows = (scope) =>
  scope.rows === 256 && scope.appeared === 128 && scope.disappeared === 128;

const gates = {
  exactAuthority:
    exactRows(entitySet)
    && exactRows(attributeScope)
    && receipt.import.result.stats.headerNodeCount === 1_000_256,
  coldPaysPageFaults:
    entitySet.coldMs > entitySet.warmP50Ms
    && attributeScope.coldMs > attributeScope.warmP50Ms,
  coldLatency: entitySet.coldMs <= 250 && attributeScope.coldMs <= 250,
  warmLatency: entitySet.warmP95Ms <= 16 && attributeScope.warmP95Ms <= 16,
  rss:
    receipt.scopes.entity_set.rss.peakDeltaBytes <= 1024 * 1024 * 1024
    && receipt.scopes.attribute_scope.rss.peakDeltaBytes <= 1024 * 1024 * 1024,
  pss:
    receipt.scopes.entity_set.pss.peakDeltaBytes <= 1024 * 1024 * 1024
    && receipt.scopes.attribute_scope.pss.peakDeltaBytes <= 1024 * 1024 * 1024,
};
for (const [name, value] of Object.entries(gates)) {
  expect(receipt.gates?.[name] === value, `${name} gate mismatch`);
  expect(value, `${name} gate failed`);
}
expect(receipt.admissionEligible === true, "admission eligibility mismatch");
expect(
  receipt.admitted === Object.values(gates).every(Boolean),
  "admission verdict mismatch",
);
expect(receipt.admitted, "browser valid-time diff not admitted");
process.stdout.write("browser valid-time diff receipt valid: admitted=true\n");
