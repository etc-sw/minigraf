import fs from "node:fs";

const paths = process.argv.slice(2);
if (paths.length === 0) {
  fail("usage: validate-browser-current-open-segment-receipt.mjs <receipt.json> [...]");
}

for (const path of paths) {
  const receipt = JSON.parse(fs.readFileSync(path, "utf8"));
  expect(receipt.schema === "vicia.browser-current-open-segment-matrix.v1", `${path}: schema`);
  expect(
    receipt.mode === "desktop-headless" || receipt.mode === "simulated-mobile",
    `${path}: mode`,
  );
  expect(receipt.admissionEligible === false, `${path}: non-admission evidence`);
  expect(receipt.source?.trackedClean === true, `${path}: clean source`);
  expect(/^[0-9a-f]{40}$/.test(receipt.source?.commit), `${path}: source commit`);
  expect(
    JSON.stringify(receipt.configuration?.segmentTargets) ===
      JSON.stringify([1, 100, 500, 1024]),
    `${path}: segment targets`,
  );
  expect(receipt.configuration?.runs === 20, `${path}: sample count`);
  expect(receipt.cases?.length === 4, `${path}: case count`);

  for (const [index, segmentCount] of [1, 100, 500, 1024].entries()) {
    const sampleCase = receipt.cases[index];
    expect(sampleCase?.segmentCount === segmentCount, `${path}: case ${segmentCount}`);
    expect(
      sampleCase.visibleDeltaPages === segmentCount,
      `${path}: one visible page per segment at ${segmentCount}`,
    );
    expect(sampleCase.opens?.length === 20, `${path}: opens at ${segmentCount}`);
    expect(
      sampleCase.summary?.sampleCount === 20 &&
        sampleCase.summary?.clockCoherentSampleCount >= 19,
      `${path}: stage clock coverage at ${segmentCount}`,
    );
    for (const sample of sampleCase.opens) {
      const physical = sample.result?.receipt;
      expect(
        physical?.schema === "vicia.browser-current-open-stage-receipt.v1" &&
          physical.outcome === "ready_current" &&
          physical.source_format_version === 12 &&
          physical.resulting_format_version === 12,
        `${path}: current-format receipt at ${segmentCount}`,
      );
      expect(
        physical.visible_delta_segment_count === segmentCount &&
          physical.visible_delta_pages === segmentCount,
        `${path}: physical lineage at ${segmentCount}`,
      );
    }
  }

  expect(
    receipt.cases[3].growth?.result?.finalAdvice === "reduce_checkpoint_cadence",
    `${path}: soft-threshold advice`,
  );
  expect(
    Object.values(receipt.gates ?? {}).every((value) => value === true),
    `${path}: self-check gates`,
  );
  console.log(
    `browser current-open segment receipt OK: ${receipt.mode} ` +
      `${receipt.cases.map((entry) =>
        `${entry.segmentCount}=${entry.summary.wallOpenMs.p95}ms`
      ).join(" ")}`,
  );
}

function expect(condition, label) {
  if (!condition) fail(label);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
