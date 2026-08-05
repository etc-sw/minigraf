# Browser current-open segment lineage risk probe

This receipt isolates one physical variable: the number of visible delta
segments loaded by `BrowserInteractiveLedger.openCurrent()`. A one-fact compact
v12 base is created first, then one-fact atomic writes build exact lineages of
1, 100, 500, and 1,024 segments. Every sample opens a fresh page and WASM
context, verifies the typed content-free receipt, and performs one bounded read
of the newest fact.

This is not release-admission evidence. The narrow run emulates a mobile
viewport, touch, a 512 MiB JavaScript heap limit, and 6x CPU throttling on the
desktop host. It does not reproduce Android process lifecycle, storage,
thermal behavior, memory pressure, or a physical device's IndexedDB/WASM
implementation.

## Environment

- Source: `25c26b90346ccc9010484bb13d71918f49eb1b2b`, tracked clean
- Browser: Chrome for Testing 150.0.7871.115, Linux headless on WSL2
- Host: AMD Ryzen 7 7800X3D, 16 logical CPUs, 32 GiB host memory
- Samples: 20 fresh page/WASM contexts per segment count and mode
- Physical shape: one visible page per segment; no projection catalog

## Results

| Visible segments | Desktop wall p50 / p95 | Desktop segment-load p95 | Simulated-mobile wall p50 / p95 | Simulated-mobile segment-load p95 |
|---:|---:|---:|---:|---:|
| 1 | 7.8 / 8.3 ms | 1 ms | 61.5 / 73.2 ms | 4 ms |
| 100 | 40.7 / 42.7 ms | 33 ms | 262.6 / 294.8 ms | 211 ms |
| 500 | 178.5 / 185.4 ms | 174 ms | 953.3 / 1,027.3 ms | 934 ms |
| 1,024 | 390.3 / 427.1 ms | 404 ms | 1,837.8 / 1,935.3 ms | 1,839 ms |

At 1,024 segments, segment loading accounts for about 95% of p95 current-open
time in both modes. IndexedDB connection, page-0 decode, and manifest metadata
remain comparatively small. The final write returns
`reduce_checkpoint_cadence`, and all 160 opens preserve exact current format,
segment count, visible pages, and newest-fact proof.

One simulated-mobile 1,024-segment sample experienced a WSL wall-clock
correction while open was in flight. Its monotonic wall measurement was
1,837.8 ms while the `Date.now()`-based stage receipt reported 5,452 ms. The
raw sample is preserved; stage summaries use the 19 samples whose receipt total
agreed with monotonic wall time within 100 ms. All other cases retained 20/20
clock-coherent stage samples.

## Judgment

This is enough to choose the next risk probe: current open is still proportional
to accumulated visible segment payload, and segment loading is the owning
stage. A bounded segment directory plus generation-pinned page-on-demand reads,
or another design with the same measured effect, is the structurally relevant
next Vicia investigation. Vetch query reduction cannot remove this physical
open cost.

The simulated-mobile 1,024-segment p95 exceeds the request's provisional
1,000 ms real-mobile budget, but it cannot fail that gate because it is not a
real device. A physical Android run is not required to continue the storage
risk probe. It is required before claiming the real-mobile budget, the reported
minute-scale stall, or the production mobile fix is closed.

Raw receipts: [desktop.json](desktop.json) and
[simulated-mobile.json](simulated-mobile.json).
