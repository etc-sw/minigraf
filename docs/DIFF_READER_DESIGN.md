# Bi-Temporal Diff Reader Design (D0)

Status: design note, no implementation. Owner lane: Claude (vicia-db). Written
2026-07-18 against source `4c01b1d`.

This note is the A-1 slice of the diff-reader line recorded in the
2026-07-18 session handoff: a bounded read primitive that returns only the
changes between two points in time, as the base material for the
vetch-memory `recall_diff` tool (Codex lane) and for any "what changed since
my last brief" consumer.

Downstream tool semantics belong to the vetch-memory lane. This note decides
only the database primitive. Choices marked **proposed** govern future work
and are canon candidates, not accepted canon.

## 1. Problem

Consumers that maintain an agent brief need "what changed between time A and
time B" without paying either of the current options:

- `export_fact_log()` / `export_fact_log_since(since_tx_count)` return an
  unbounded `Vec<FactRecord>` with no entity/attribute scope and no paging.
  They are audit/replay surfaces, not foreground reads.
- Running two full Datalog reads at two `:as-of`/`:valid-at` points and
  diffing client-side pays the scope twice and re-implements retraction and
  window semantics in every caller.

The primitive must obey the existing foreground read discipline: bounded,
fail-closed instead of truncating, transaction-pinned, no hidden
O(total-history) work.

## 2. Semantic Model

A bi-temporal ledger has two time axes, so "diff between two times" has three
distinct meanings. The design must name all three so the API never blurs them:

| Axis | Question | Cost owner |
| --- | --- | --- |
| D-tx: transaction axis | Which ledger records were committed in `(tx_a, tx_b]`? | tx tail (A2 page probe: O(log pages) + O(tail)) |
| D-valid: valid-time axis | At one fixed tx basis, how does visible truth at `valid_at = t2` differ from `valid_at = t1`? | selected scope's index range |
| D-both: both axes | Difference between two full `(tx, valid)` coordinates. | superset; out of scope |

D-tx is a record diff: it reports writes, including retraction records, in
commit order. It answers "what happened since my last session" but reports a
backdated correction only as an opaque new record.

D-valid is a truth diff: it reports net visibility changes between two valid
times as currently recorded. It is the semantics the handoff names
("두 valid-time 사이 변경만 반환") and the one a memory brief actually wants:
it catches retroactive corrections (backdated `valid_from`), which are the
reason vetch-memory is bi-temporal at all.

### Decision (proposed)

- **v0 implements D-valid** as a `ReadView` primitive: `t1`/`t2` on the valid
  axis, tx basis pinned by the view. This is the recall_diff floor.
- **D-tx is deferred to a v0.5 candidate**: a bounded, scoped, paged variant
  of the existing `export_fact_log_since` tail read. The unbounded export
  APIs already cover audit; v0.5 exists only if a caller proves the bounded
  foreground shape is needed. Do not implement it inside the v0 slice.
- **D-both is a non-goal** until a consumer states a concrete question that
  needs it.

### D-valid definition

Within one transaction-pinned view (tx basis `T`), for valid instants
`t1 < t2`, using the existing half-open visibility rule
`visible(t) := valid_from <= t < valid_to` over net-asserted windows at basis
`T` (scoped and unscoped retractions applied exactly as today):

- For each `(entity, attribute)` in scope, compute the visible value set at
  `t1` and at `t2`.
- Emit one row per set difference element:
  - `Appeared`: value visible at `t2`, not at `t1`.
  - `Disappeared`: value visible at `t1`, not at `t2`.
- A value replacement is one `Disappeared` plus one `Appeared` row for the
  same `(entity, attribute)`; the consumer pairs them. No synthetic
  `Changed` kind in v0 — multi-valued attributes make pairing a consumer
  policy, not a storage fact.

Explicit v0 non-goal: **net semantics only.** A value that appears and then
disappears strictly inside `(t1, t2)` is invisible to D-valid by definition.
Interval-internal churn is D-tx material (audit records), not a truth diff.

Boundary semantics are inherited, not invented: `t1` and `t2` use the same
half-open `interval_contains` rule as the current-projection reducer
(`src/graph/current_projection.rs:859`). A window with `valid_from == t2` is
visible at `t2`; a window with `valid_to == t2` is no longer visible at `t2`.
No new boundary convention is introduced.

## 3. Scope Rules

D-valid cost is owned by the selected index range. There is no valid-time-keyed
index (valid_from/valid_to appear inside EAVT/AEVT/AVET/VAET keys after the
entity/attribute/value prefix, `src/storage/index.rs:102-393`), and v0 must not
add one. Therefore:

- v0 scope is **one attribute plus an optional explicit entity set**, mirroring
  `CurrentEntitiesRequest`:
  - entity set present: exact EAVT `(entity, attribute)` ranges, up to the
    existing 128-entity selective ceiling;
  - entity set absent: one AEVT `(attribute)` range.
- An **unscoped whole-database diff is rejected** at request validation. That
  shape is O(total-history) and belongs to export/maintenance surfaces, not a
  foreground read view.

## 4. Public API Shape (proposed)

Native, on the existing transaction-pinned view (`src/db.rs:468`), reachable
only through `InteractiveLedger::read_view()` — the `MaintenanceLedger` split
keeps diff reads out of maintenance lifetimes by construction:

```rust
pub struct ValidTimeDiffRequest<'a> {
    pub attribute: &'a str,
    pub entities: Option<&'a [EntityId]>, // None => AEVT attribute range
    pub valid_at_before: i64,             // t1, Unix ms
    pub valid_at_after: i64,              // t2, Unix ms; must be > t1
    pub after: Option<&'a str>,           // opaque continuation
    pub limit: usize,                     // <= READ_VIEW_MAX_ROWS
}

pub enum ValidTimeDiffChange { Appeared, Disappeared }

pub struct ValidTimeDiffRow {
    pub entity: EntityId,
    pub attribute: String,
    pub value: Value,
    pub change: ValidTimeDiffChange,
    // Provenance: the window that makes the row true on its visible side
    // (the t2-covering window for Appeared, the t1-covering window for
    // Disappeared).
    pub valid_from: i64,
    pub valid_to: i64,
}

pub struct ValidTimeDiffPage {
    pub rows: Vec<ValidTimeDiffRow>,
    pub continuation: Option<String>,
}

impl ReadView<'_> {
    pub fn valid_time_diff(
        &self,
        request: ValidTimeDiffRequest<'_>,
    ) -> Result<ValidTimeDiffPage>;
}
```

Contract rules, all inherited from the existing bounded-read discipline:

- The view must be constructed with `ReadViewValidAt::AnyValidTime`, exactly
  like `entity_attribute_history`: the request supplies both valid instants
  itself, so a valid-time-pinned view would be a contradiction.
- Bounds reuse the existing constants (`src/db.rs:81-104`): complete pages cap
  at `READ_VIEW_MAX_ROWS` (10,000) rows, `65,536` source index entries, and
  `8 MiB` encoded result per page, stepping `4,096` entries at a time.
  Exceeding a bound with an incomplete `(entity, attribute)` group fails
  closed; there is no silent truncation.
- Continuations reuse the CRC-protected opaque-cursor scheme of
  `entity_attribute_history` (`src/db.rs:274-351`): version + CRC validated,
  bound to `(attribute, entity-set hash, t1, t2, view tx_count)`, resuming at
  the last completed EAVT/AEVT key. A cursor presented against a different
  request or view is rejected.
- Pending (uncheckpointed same-handle) facts follow whatever
  `entity_attribute_history` does at the same tx basis — the diff must not
  invent a new pending-visibility rule. Verify and pin with a test rather
  than assuming.

Browser mirror (`src/browser/mod.rs`): `BrowserReadView.validTimeDiff()`
async, same bounds plus the browser-wide `8 MiB` structured-result bound,
paged-generation checks and sparse page demand identical to
`entityAttributeHistory()`.

Naming is open (§8): `valid_time_diff` vs `changes_between`. The name must
carry the axis, because a D-tx surface may exist later.

## 5. Algorithm Sketch and Reuse Map

One pass over the selected scope range; no snapshot materialization, no
second scan:

```text
for each (entity, attribute) group in EAVT/AEVT scope order:
    reduce the group's history at the view tx basis into net-asserted
      visible windows            # existing retraction/window semantics
    values_at_t1 := { value | some window contains t1 }
    values_at_t2 := { value | some window contains t2 }
    emit Disappeared rows for values_at_t1 - values_at_t2
    emit Appeared   rows for values_at_t2 - values_at_t1
```

| Need | Existing surface | Where |
| --- | --- | --- |
| Group-ordered scoped scan, tx pin, retraction semantics | `entity_attribute_history` scan machinery | `src/db.rs:646-704` |
| Per-entity visible-window enumeration | interval cursor with `emit_each_visible_window` | `src/graph/storage.rs:2139-2154` |
| Instant-in-window test | `interval_contains` | `src/graph/current_projection.rs:859` |
| Budget stepping and fail-closed overflow | history paging constants and page finishing | `src/db.rs:81-104, 388-432` |
| Opaque continuation encode/decode | history cursor CRC scheme | `src/db.rs:274-351` |
| 128-entity exact-set ceiling | selective-read entity budget | existing no-full-scan coverage |

Per-group state is bounded by that group's history, which is already capped by
the source-entry budget. Nothing new is retained across groups except the
continuation position.

## 6. Correctness Matrix (A-2 tests-first)

Applicable rows from the roadmap correctness matrix, plus diff-specific rows:

- appeared / disappeared / replaced single-value cases;
- multi-valued attribute: partial set change emits only the changed values;
- scoped retraction that truncates a window across `t1` or `t2`;
- unscoped retraction (`RETRACT_ALL_VALID_FROM`) before/between/after
  `t1`/`t2`;
- boundary exactness: window with `valid_from == t2` (visible at t2), window
  with `valid_to == t2` (not visible at t2), same at `t1`; `t1 >= t2`
  rejected;
- interval-internal churn (appear+disappear strictly inside `(t1, t2)`)
  emits nothing — pins net semantics;
- backdated correction: a fact committed recently with old `valid_from`
  appears in a diff over old instants — pins the reason D-valid exists;
- all `Value` variants including `Ref` (VAET untouched but Ref rows must
  round-trip) and `Keyword`;
- base-only, delta-only, base+delta merge, same-key base/delta ordering;
- view tx pinning: facts committed after the view's `tx_count` are invisible
  to the diff; pending-fact behavior pinned to match
  `entity_attribute_history`;
- continuation resume across a page boundary with no duplicate and no
  missing row; cursor rejected against a different request/view;
- over-budget group fails closed (row cap, source-entry cap, byte cap);
- unscoped-database request rejected;
- entity-set request stays on the selective committed-index path
  (no-full-scan regression, native and real-Chrome paged);
- native/WASM result parity on a shared fixture.

## 7. Gates for A-2 (implementation slice)

- All matrix rows green; existing suites untouched and green.
- A clean 1M receipt in `docs/BENCHMARKS.md`:
  - 128-entity scoped diff p95 in the same order as `entity_attribute_history`
    point paging (sub-ms target), with zero full-image decode and no measured
    query RSS growth;
  - attribute-scoped diff cost proportional to the attribute's index range,
    never to total history.
- `cargo fmt --check`, `cargo test`, `cargo clippy --lib -- -D warnings`,
  `git diff --check` (all-target clippy stays excluded per the pre-existing
  test-lint debt caveat).
- Browser slice (A-3) additionally: wasm32 build, real-Chrome regression for
  the new method, package gates only via the standard cross-cutting sequence
  (clean native receipt → Chrome matrix → Vetch vendor sync). No package
  publish from this line without that full sequence.

## 8. Open Questions (state after A-2)

1. **Naming**: resolved — `valid_time_diff`, encoding the valid axis in the
   method name.
2. **Row provenance payload**: resolved for v0 — one covering window per row.
   Adding further fields later is additive.
3. **Pending-fact visibility**: resolved — pending same-handle facts are
   visible under the view tx basis, matching `entity_attribute_history`
   (pinned by the committed/delta/pending layering test).
4. **Page-break semantics**: resolved — pages break at entity-group
   boundaries; a continuation resumes at the first unemitted group, which is
   re-scanned with a fresh source budget. A single group that alone exceeds
   the row limit or source budget fails closed on its own page.
5. **v0.5 D-tx surface**: still open — bounded scoped paged tail read on the
   A2 probe; open only if vetch-memory's resume/attention flows prove the
   unbounded `export_fact_log_since` is actually hot in a foreground path.
6. **recall_diff mapping**: still vetch-memory-lane design; the contract this
   note fixes is the row shape and net semantics above.

## 9. Philosophy and Compatibility Check

- Embedded, single-file, Datalog-first: unchanged. This is a typed bounded
  read on an existing view, not a query language or server feature.
- No new index, no file-format change, no new dependency, no maintenance
  coupling; WAL/publication/recovery rules untouched.
- Public API growth is additive on `ReadView`/`BrowserReadView` under the
  existing 1.x compatibility policy. `#[non_exhaustive]` is applied per
  direction, not uniformly — A-2 resolved this after finding that a blanket
  application would have made the request unconstructible:

  | Type | Direction | `#[non_exhaustive]` | Why |
  | --- | --- | --- | --- |
  | `ValidTimeDiffChange` | out | yes | A later change kind must not break an exhaustive `match`. |
  | `ValidTimeDiffRow` | out | yes | §8 promises further provenance fields stay additive. |
  | `ValidTimeDiffPage` | out | yes | Page-level metadata is a plausible later addition. |
  | `ValidTimeDiffRequest` | **in** | **no** | Callers build it with a struct literal. Marking it would make it unconstructible outside the crate without a builder, and the sibling `CurrentEntitiesRequest` / `EntityAttributeHistoryRequest` are plain structs. Adding a required field is a breaking change either way; adding an optional one needs a builder, which is the moment to introduce it. |
  | `ValidTimeDiffCursor` | opaque | not needed | Every field is `pub(crate)`, so external construction and exhaustive destructuring are already impossible. The attribute would be a no-op. |
- Bi-temporal first-class: this primitive is an argument *for* the
  philosophy — it is exactly the read that only a bi-temporal ledger can
  answer, and it makes the valid axis useful to agent consumers instead of
  decorative.

## 10. Slice Plan

| Slice | Content | Exit |
| --- | --- | --- |
| A-1 (done) | Semantics, scope, API shape, reuse map, matrix, gates. | Note committed; cross-lane decision recorded in vetch-memory as proposed. |
| A-2 (done) | Native `valid_time_diff` tests-first per §6, receipt per §7. | 17-test matrix green in `tests/valid_time_diff_test.rs`; 1M measurement receipt in `docs/BENCHMARKS.md`. |
| A-3 | Browser mirror + real-Chrome regression; package only via standard sequence. | Chrome suite green. |
| A-4 | Handoff to Codex lane: capability transition + refs so recall_diff can build. | vetch-memory records updated; no vicia-db code. |
