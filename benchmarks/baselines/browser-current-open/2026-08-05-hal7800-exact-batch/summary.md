# Browser current-open exact-page batch result

This receipt reruns the visible-segment cold-open matrix after replacing one
IndexedDB readonly transaction per segment range with bounded exact-page
batches. Each batch reads page 0 once, requests only named segment pages, and
returns independently valid pages. The caller reconstructs complete segment
ranges before the unchanged persistent manifest selector chooses the newest
valid generation or falls back to its predecessor.

The source is clean commit `c476e8f763a466cab0233d9c0fd6a9fb0c2a162c`.
The browser package was built with `--features browser`; the measurement does
not require `bench-internals`.

## Environment

- Browser: Chrome for Testing 150.0.7871.115, Linux headless on WSL2
- Host: AMD Ryzen 7 7800X3D, 16 logical CPUs, 32 GiB host memory
- Samples: 20 fresh page/WASM contexts per segment count and mode
- Physical shape: one visible page per segment; no projection catalog
- Baseline: `../2026-08-05-hal7800-risk-probe/`

## End-to-end p95

| Visible segments | Desktop baseline | Desktop exact batch | Improvement | Simulated-mobile baseline | Simulated-mobile exact batch | Improvement |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 8.3 ms | 8.2 ms | 1.2% | 73.2 ms | 57.4 ms | 21.6% |
| 100 | 42.7 ms | 16.8 ms | 60.7% | 294.8 ms | 92.9 ms | 68.5% |
| 500 | 185.4 ms | 36.5 ms | 80.3% | 1,027.3 ms | 233.4 ms | 77.3% |
| 1,024 | 427.1 ms | 76.2 ms | 82.2% | 1,935.3 ms | 380.5 ms | 80.3% |

At 1,024 segments, desktop segment-load p95 falls from 404 ms to 46 ms
(88.6%), and simulated-mobile segment-load p95 falls from 1,839 ms to 269 ms
(85.4%). Desktop p95 sampled PSS growth falls from 59,386,880 to 36,739,072
bytes. Simulated-mobile p95 sampled PSS growth falls from 37,571,584 to
34,835,456 bytes.

## Gate judgment

The risk probe passes. Both 500- and 1,024-segment p95 cases improve by more
than the required 20% in both modes. All 160 opens preserve the current format,
exact visible segment and page counts, selected generation, and newest-fact
proof. The real-Chrome structural suite passes all 93 tests, including corrupt
candidate recovery. Exact-page batching is retained as a durable browser
backend primitive.

This does not make open asymptotically bounded: page fetch, segment decode, and
resident construction still scale with visible segment payload. At 1,024
segments the remaining simulated-mobile p95 is 269 ms in segment loading and
95 ms in persistent construction. A generation-pinned lazy segment directory
remains the larger direction if further cold-open work is justified.

One simulated-mobile 1,024-segment sample has a stage-clock disagreement; the
raw sample is retained, and the stage summary uses the other 19 coherent
samples. This remains simulated-mobile risk evidence, not a physical-device
release gate.

Raw receipts: [desktop.json](desktop.json) and
[simulated-mobile.json](simulated-mobile.json).
