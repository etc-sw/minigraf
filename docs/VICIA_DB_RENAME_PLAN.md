# Vicia DB Rename Plan

Status: V0–V2 complete. **V3 decided on 2026-07-27: publish to crates.io as
`vicia-db`, starting at `0.1.0`.** The V3 gate is closed (see "V3 Decision"
below); the package/type rename itself has not been performed yet, and no file
format or language-binding rename has happened.

Date: 2026-06-06, V3 decision 2026-07-27

Branch: `vicia/api-alias` for the V2 update. The initial plan landed on
`vicia/rename-plan`. The V3 decision landed on `vicia/rename-v3`.

## Recommendation

Adopt **Vicia DB** as the Vetch-oriented name for this Minigraf line, but do it
as a staged successor/fork rename rather than a broad in-place rewrite.

The first implementation steps are complete: docs/metadata introduced the
Vicia DB transition, and the Rust API exposes `ViciaDb` as a compatibility alias
for `Minigraf`. The publish decision that gated the package rename was taken on
2026-07-27 — see "V3 Decision" — so the package and type rename is now cleared
to proceed. Language-binding renames stay behind it.

## Name Rationale

`Vicia` is the botanical genus associated with vetch. That gives the database a
direct relationship to Vetch without using Earthsea-specific names or making the
brand feel derivative.

Preferred naming:

| Surface | Preferred Name | Notes |
| --- | --- | --- |
| Product / docs | Vicia DB | Human-facing name. |
| Rust package | `vicia-db` | Hyphenated package name if published separately. |
| Rust crate import | `vicia_db` | Rust import form. |
| Decision skill | `vicia-db-decision-gate` | Reusable decision workflow for storage/read-path gates. |
| File extension | Keep `.graph` initially | Avoid file-format churn during rename. |

## Relationship To Minigraf

Vicia DB should be described as a Vetch-oriented successor/fork of Minigraf:

> Vicia DB is a Vetch-oriented successor of Minigraf: an embedded, single-file,
> bi-temporal graph ledger optimized for local-first agent context.

This framing keeps the technical lineage clear while explaining why the name
changes. The rename is not only cosmetic: Vetch now imposes concrete operating
constraints around 1M+ local facts, receipt-sized writes, agent-brief read
latency, full-history identity, and background maintenance.

## Philosophy Fit

The rename is acceptable only if it preserves the existing Minigraf philosophy:

- embedded-first library, not a server
- single durable `.graph` file
- Datalog remains the query language
- no new dependencies for branding
- file-format stability remains more important than name consistency
- API compatibility is protected during the transition

The rename must not become an excuse to add vector search, BM25, multimodal blob
storage, sidecar indexes, or a client/server layer to the core database. Those
remain Vetch-side projections until a benchmark-backed proposal proves they
belong in the core.

## Compatibility Policy Result

V2 chose the lowest-risk option before any package or binding rename work:

| Option | Shape | Tradeoff |
| --- | --- | --- |
| Alias-first | Add `ViciaDb` as a public alias/wrapper while keeping `Minigraf`. | Lowest breakage, slightly awkward dual naming. |
| Type rename with alias | Rename primary type to `ViciaDb`, keep `type Minigraf = ViciaDb` for one compatibility window. | Clearer new identity, more doc and example churn. |
| Hard rename | Remove `Minigraf` public type. | Cleanest brand, highest compatibility risk. Not recommended yet. |

Current result: **alias-first** is implemented. `ViciaDb` is a public type alias
for `Minigraf`, preserving the existing API, package name, and file format.

V3 selected the next step: **type rename with alias**. The package rename is the
break-window this was waiting for, so R3 promotes `ViciaDb` to the primary type
and keeps `Minigraf` as `pub type Minigraf = ViciaDb`. Hard rename — removing
`Minigraf` — is still not recommended.

## Rename Surfaces

The rename touches more than `Cargo.toml`.

| Surface | First Action | Later Action |
| --- | --- | --- |
| `Cargo.toml` | Decide package/repository/documentation URLs. | Rename package only when publish path is ready. |
| README | Introduce Vicia DB as successor/fork. | Replace examples after API policy is chosen. |
| Rust API | Add `ViciaDb` alias (done in V2). | Consider primary type rename only after a publish or break-window decision. |
| Docs | Add rename plan and update roadmap references. | Replace user-facing Minigraf naming where appropriate. |
| Tests/benches | None initially. | Rename imports after crate/package decision. |
| Language bindings | Defer. | Treat as separate releases with compatibility notes. |
| Wiki | Defer until repo rename is real. | Update after final doc sync. |
| License | Add missing license files and attribution before publishing. | Keep original notices and fork lineage. |
| GitHub/crates/docs.rs | No change until publish decision. | Create/rename only after compatibility gate. |

## License And Attribution Checklist

Before publishing a Vicia DB fork/package:

- Keep the actual `LICENSE-MIT` and `LICENSE-APACHE` files in the checkout.
- Preserve original copyright and license notices.
- State that Vicia DB is derived from/forked from Minigraf.
- Keep `MIT OR Apache-2.0` unless there is a deliberate legal reason to change.
- Do not imply endorsement by the original Minigraf project or organization.
- If the Apache-2.0 path is used, preserve any required NOTICE material.

This document is not legal advice; it is an engineering checklist for avoiding
avoidable open-source hygiene mistakes.

## Decision Skill Candidate

Skill name: `vicia-db-decision-gate`

Purpose:

- Reduce repeated judgment cost when deciding storage, read-path, checkpoint,
  recompact, or public API changes for Vicia DB.

Trigger examples:

- "Vicia DB checkpoint/read path/storage decision"
- "Vetch 1M baseline affects Vicia DB"
- "Should Vicia DB add an API/index/recompact behavior?"
- "Review this Vicia DB storage plan"

Core output shape:

```text
Recommendation:
Risk:
Required gate:
First slice:
Verification:
Rejected:
```

Core rules:

- Measure before optimizing.
- Keep benchmark slices separate from implementation slices.
- Preserve full-history identity:
  `entity`, `attribute`, encoded `value`, `valid_from`, `valid_to`,
  `tx_count`, `tx_id`, and `asserted`.
- Treat `Value::Ref` as mandatory coverage for Vetch graph/ledger behavior.
- Keep small write/checkpoint cost tied to pending/delta size, not committed
  graph size.
- Keep recompact idle/background/scheduled.
- Add public APIs only after the measured Vetch path proves that internal
  storage/query work is insufficient.
- Use reference databases for invariants, not dependencies.
- Separate fallback-safe recovery states from error states before changing
  publish or WAL-retire behavior.

Do not put volatile benchmark numbers directly in the skill. The skill should
route agents to `docs/BENCHMARKS.md`, `docs/DELTA_INDEX_DESIGN.md`, and
`docs/internal/VETCH_DELTA_STORAGE_ROADMAP.md` for current evidence.

## Proposed Slice Plan

### V1 Completion

V1 introduces Vicia DB naming without changing code:

- README transition note.
- Storage roadmap link to this rename plan.
- Delta design link to this rename plan.
- License files confirmed present.

### V0: Rename Plan

This slice.

Done when:

- `docs/VICIA_DB_RENAME_PLAN.md` exists.
- No code/package/API rename has happened.
- The next slice is explicit.

### V1: Docs And Metadata Preparation

Goal:

- Introduce Vicia DB naming without breaking code.

Allowed:

- README wording update.
- Roadmap/design docs update.
- License file addition.
- Attribution wording.
- Optional badges removed or marked pending if URLs are not real yet.

Forbidden:

- Public Rust API break.
- File-format version change.
- Language binding rename.
- New dependencies.

Verification:

- `git diff --check`
- `cargo test` if any doctest/example text changes compile against Rust APIs.

### V2: Rust API Compatibility Alias

Goal:

- Add `ViciaDb` while preserving `Minigraf` compatibility.

Allowed:

- Public alias/wrapper.
- Docs/examples showing Vicia DB first.
- Tests proving `Minigraf` and `ViciaDb` open/query/checkpoint equivalently.

Forbidden:

- Removing `Minigraf`.
- Changing file extension or file format.

Verification:

- Targeted API compatibility tests.
- `cargo test`
- `cargo clippy --lib -- -D warnings`
- `cargo fmt -- --check`

Result:

- Done: `ViciaDb` is exported as a public type alias for `Minigraf`.
- Done: tests prove in-memory usage through `ViciaDb`, legacy `Minigraf`
  interoperability, file-backed checkpoint, and reopen through `Minigraf`.
- Done: no package rename, file-format change, or binding rename.

### V3: Package/Repository Publish Decision

Goal:

- Decide whether to publish as `vicia-db`, rename repository, or keep a Vetch
  internal fork.

Gate:

- License files present.
- Attribution text present.
- Downstream package/binding impact listed.
- Name availability checked for the actual publish targets.

## V3 Decision (2026-07-27)

Decision: **publish to crates.io as `vicia-db`**, and promote `ViciaDb` to the
primary Rust type in the same window, keeping `Minigraf` as the compatibility
alias. The package rename is the only break-window this line gets, so the type
flip rides along with it rather than costing a second round of doc and example
churn.

| Item | Decision |
| --- | --- |
| Crate name | `vicia-db` (import form `vicia_db`) |
| First version | `0.1.0` — not a continuation of upstream's 1.1.1, which never described this fork's v10–v13 storage |
| `authors` | `["etc-sw"]` — who ships the package. Upstream copyright stays in `LICENSE-MIT`; lineage stays in README |
| `repository` | `https://github.com/etc-sw/vicia-db` |
| `documentation` | Omitted until the first publish creates a real docs.rs page |
| Primary type | `ViciaDb`, with `pub type Minigraf = ViciaDb` |
| File format | Unchanged. `.graph`, `MGRF`, `MGCPG001`, `MGDMF001`, `MGDSG001`, and every format version stay as-is |

### Gate results

- **Name availability (checked 2026-07-27).** crates.io has no `vicia-db`,
  `vicia_db`, or `vicia`. npm has no `@vicia-db/browser`. The names are free
  but unreserved — nothing has been published to hold them.
- **License files.** `LICENSE-MIT` (© 2023-2026 Aditya Mukhopadhyay) and
  `LICENSE-APACHE` are present and unmodified. No `NOTICE` file exists
  upstream, so Apache-2.0 §4(d) adds no obligation here.
- **Attribution.** README carries the fork banner, the upstream link, and the
  lineage statement under "License".

### Downstream impact

Three local consumers. They fail differently, which sets the landing order.

| Consumer | Edge | Breaks on rename? | Fix |
| --- | --- | --- | --- |
| `~/projects/vetch-memory` | `Cargo.toml: minigraf = { path = "../vicia-db" }`, 33 `minigraf::` references across 18 `.rs` files | **Yes, immediately.** The path dependency points at this repo's main checkout, so it breaks the moment the package renames | One line: `minigraf = { package = "vicia-db", path = "../vicia-db" }`. Zero source changes. Sweep the 18 files to `vicia_db::` later, on vetch-memory's own schedule |
| `~/projects/being-public` | `config/vicia-source.json` pins `cargo_package: "minigraf"`, `binary: "minigraf"`, and a git rev | No — it builds a pinned rev, so it keeps working until it bumps | Bump `rev` + `cargo_package` + `binary` together in one commit. Never bump the rev alone |
| `~/projects/vetch-app` | `@vicia-db/browser` via `link:vendor/vicia-browser` | No — already insulated. `scripts/sync-vetch-browser-package.sh` passes `--out-name vicia_db`, so the vendored artifacts are already `vicia_db.js` / `vicia_db_bg.wasm` | None |

The `package = "vicia-db"` rename-alias pattern is already proven in this repo:
`bindings/browser/Cargo.toml` uses `vicia_db = { package = "minigraf", path = "../.." }`
today, in the opposite direction.

### Landing order

- **R1 — docs/metadata.** README title, `Cargo.toml` `authors`/`repository`/
  `documentation`. No code. This slice.
- **R2 — this section.** The decision recorded before anything depends on it.
- **R3 — package + type rename, one commit.** `name = "vicia-db"`,
  `version = "0.1.0"`, lib `vicia_db`, `ViciaDb` promoted with `Minigraf` as
  alias, `use minigraf::` → `use vicia_db::` across tests/benches/examples,
  `benches/minigraf_bench.rs` renamed, CLI binary and `libminigraf.so` names.
  Must include, or CI lies: `.github/workflows/binary-size.yml:37,52` hardcode
  `target/release/libminigraf.so` and `target/release/minigraf` — rename
  without them and the size budget silently measures a file that no longer
  exists. Also `justfile:158` validates `{{OUTPUT_DIR}}/minigraf.d.ts`, and the
  separate workspaces `fuzz/`, `tools/cross-db-bench`, `tools/ref-db-bench`,
  `bindings/browser` all depend on the package by name. Land the vetch-memory
  one-liner in the same session.
- **R4 — leave history alone.** `CHANGELOG.md` entries and the 91 files under
  `docs/superpowers/plans|specs` keep saying `minigraf`, because they record
  what was true then. A blanket rewrite would erase that.
- **R5 — re-arm release CI, separate commit, last.** `release.yml`
  `-p minigraf` → `-p vicia-db`, `cascade.yml` dispatch targets moved off
  `project-minigraf/*`. The `push: tags:` trigger and `CARGO_REGISTRY_TOKEN`
  are added *here and nowhere earlier* — both files carry a DISARMED banner
  warning that adding the token mid-rename silently arms a publish. Note
  `release.yml` is cargo-dist generated; `dist generate` restores the trigger.
- **R6 — publish, then bindings (V4).** `cargo publish --dry-run`, publish,
  then npm/PyPI/Maven. Never in the same commit as core changes.

### Open risk

The names are free but unreserved. Nothing holds `vicia-db` on crates.io
between this decision and R6. Reserving it with a placeholder publish is an
irreversible public action and is not part of this slice.

### V4: Binding And Ecosystem Rename

Goal:

- Rename bindings only after the core Rust compatibility path is stable.

Gate:

- Separate checklist for npm, PyPI, Maven, Swift, C, WASM, and docs.rs.
- No binding rename should be bundled into core storage changes.

## Non-Goals

- Do not rename storage format in this planning slice.
- Do not change `.graph` files.
- Do not alter V10 delta storage behavior.
- Do not create a public recompact API as part of rename.
- Do not fold Q2-B cleanup work into rename work.
- Do not publish to crates.io/npm/PyPI from this branch.

## Open Questions

Answered by the V3 decision:

- ~~Should a future release promote `ViciaDb` to the primary struct?~~ Yes, in
  R3, with `Minigraf` as the compatibility alias.
- ~~Should Vicia DB remain Vetch-internal?~~ No — it publishes as `vicia-db`.
- ~~Which organization/repository should own the successor package?~~
  `etc-sw/vicia-db`.

Still open:

- Should `minigraf` remain as a compatibility crate that re-exports
  `vicia_db`? This fork does not own the upstream crate name, so this is only
  possible if upstream cooperates. Assume no.
- Should the public file extension remain `.graph` indefinitely?
- Does the crates.io name need reserving before R6, or is the exposure window
  short enough to accept? See "Open risk".

## Current Recommendation

Proceed to R3 — the package and type rename — as a single commit on its own
branch, landing the `~/projects/vetch-memory` one-line dependency fix in the
same session so the path dependency never sits broken. Keep Vetch
maintenance-caller validation and any binding rename work on separate branches.
