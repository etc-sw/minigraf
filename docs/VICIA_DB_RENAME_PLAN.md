# Vicia DB Rename Plan

Status: V0–V3 complete; R1–R3 landed on 2026-07-27 (merge `fa27cf6`). The
package is `vicia-db`, the import path is `vicia_db`, and `ViciaDb` is the
primary type with `Minigraf` kept as a live alias. **Publishing is on, and
manual** — `vicia-db` to crates.io and `@vicia-db/browser` to npm, released by
hand; R5 (arming release CI) is declined. See "Publish Decision, Final" below.
No file format or language-binding rename has happened.

Date: 2026-06-06, V3 decision 2026-07-27, publish deferred and then reinstated
as manual-only 2026-07-27

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

Decision: **rename the package to `vicia-db`**, and promote `ViciaDb` to the
primary Rust type in the same window, keeping `Minigraf` as the compatibility
alias. (This section originally also decided to publish to crates.io. That half
was deferred and then reinstated as manual-only the same day — see "Publish
Decision, Final". The name, version, metadata, and type decisions below stand
unchanged; they are what the rename needed, not what publishing needed.) The package rename is the only break-window this line gets, so the type
flip rides along with it rather than costing a second round of doc and example
churn.

| Item | Decision |
| --- | --- |
| Crate name | `vicia-db` (import form `vicia_db`) |
| First version | `0.1.0` — not a continuation of upstream's 1.1.1, which never described this fork's v10–v13 storage |
| `authors` | `["etc-sw"]` — who ships the package. Upstream copyright stays in `LICENSE-MIT`; lineage stays in README |
| `repository` | `https://github.com/etc-sw/vicia-db` |
| `documentation` | Omitted. docs.rs builds a page automatically from the crates.io release, and Cargo defaults to it |
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

- **R1 — docs/metadata (landed, `ea8a40b`).** README title, `Cargo.toml`
  `authors`/`repository`/`documentation`. No code.
- **R2 — this section (landed, `ea8a40b`).** The decision recorded before
  anything depends on it.
- **R3 — package + type rename, one commit (landed, `5f91c6c`; docs synced in
  `1633af9`).** `name = "vicia-db"`,
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
- **R5 — re-arm release CI. DECLINED, see "Publish Decision, Final".** `release.yml`
  `-p minigraf` → `-p vicia-db`, `cascade.yml` dispatch targets moved off
  `project-minigraf/*`. The `push: tags:` trigger and `CARGO_REGISTRY_TOKEN`
  are added *here and nowhere earlier* — both files carry a DISARMED banner
  warning that adding the token mid-rename silently arms a publish. Note
  `release.yml` is cargo-dist generated; `dist generate` restores the trigger.
- **R6 — publish, then bindings (V4). Manual, see "Publish Decision, Final".** `cargo publish --dry-run`, publish,
  then npm/PyPI/Maven. Never in the same commit as core changes.

### Open risk

The names are free but unreserved until the first release of each lands. See
"Reserved-name exposure, closed".

### V4: Binding And Ecosystem Rename

Goal:

- Rename bindings only after the core Rust compatibility path is stable.

Gate:

- Separate checklist for npm, PyPI, Maven, Swift, C, WASM, and docs.rs.
- No binding rename should be bundled into core storage changes.

## Publish Decision, Final (2026-07-27)

Decision: **publish, manually.** `vicia-db` goes to crates.io and
`@vicia-db/browser` goes to npm. R5 — arming the release CI — is **declined**;
releases are run by hand from a clean checkout.

This section records a decision that moved twice in one day, and the reasoning
for each move is worth keeping because the trade is real:

1. **Deferred first.** No consumer resolves this package from a registry —
   `vetch-memory` uses a path dependency, `being-public` pins a git revision,
   `vetch-app` links a vendored build. On that evidence a release buys nothing
   a consumer needs.
2. **Then reinstated.** The deferral left both names unreserved indefinitely,
   and there is no reservation mechanism on crates.io short of publishing.
   Publishing a real `0.1.0` from substantial code is a legitimate claim on the
   name rather than squatting, and it is the only claim available.

What survived the reversal is the *reason* the deferral was attractive: the
danger was never the release, it was the armed pipeline. So the release
happens and R5 does not.

| Item | Decision |
| --- | --- |
| `vicia-db` on crates.io | Publish `0.1.0` manually |
| `@vicia-db/browser` on npm | Publish `0.1.0` manually via `just publish-browser` |
| R5 — release CI | **Declined.** No tag trigger, no `CARGO_REGISTRY_TOKEN` in CI |
| Language bindings (PyPI/Maven/Swift/C) | Unchanged — still upstream's, still V4 |

### Why R5 stays declined

`release.yml` is cargo-dist generated and its publish job runs
`cargo publish -p <name>` on a `push: tags:` trigger. The release checklist in
CONTRIBUTING.md ends with "tag pushed". Arming that pipeline makes an ordinary
`git push origin v0.1.1` a publish, and the only thing standing between the two
is a secret that someone will eventually add for an unrelated reason. Manual
release costs one command per release and removes that failure mode entirely.

Both workflows keep their DISARMED banners, and both banners now say that
manual release is the decision rather than a step someone forgot.

Note the latent hazard the banners call out: `release.yml`'s publish job still
says `-p minigraf`, upstream's crate name. It is left wrong on purpose — with
no token the job fails either way, and a wrong name fails louder than a right
one. Do not "fix" it without deciding to arm releases.

### npm release path

`@vicia-db/browser` had no release path at all before this decision; the sync
script only swapped a staged build into Vetch's vendor directory. The npm
publish reuses that same staged package and its gates — `wasm-pack` build,
provenance stamp, browser integration gates against the sibling Vetch checkout,
and a source-stability recheck — and then runs `npm publish --access public`
instead of the vendor swap. It refuses a dirty checkout with no override.

It deliberately does **not** update Vetch's vendored package. Vetch consumes a
local `link:` build so its iteration never waits on a release, and a release
must not move Vetch underneath the person running it.

### Reserved-name exposure, closed

The earlier "Open risk" — that `vicia-db` and the `@vicia-db` scope were free
but unreserved — is closed by the first release of each. Until those land it is
still open.

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
- ~~Does the crates.io name need reserving before R6?~~ Answered: yes, and
  publishing `0.1.0` is the only way to do it. See "Publish Decision, Final".

## Current Recommendation

Nothing is pending. R1–R3 landed, all three downstream consumers were repinned
in the same session (`vetch-memory` `cac914f`, `being-public` `32c2563`;
`vetch-app` needed no change because its sync script already emits
`vicia_db`), and R4 is a decision to leave history alone rather than work to do.
R6 is a manual release of both packages; R5 is declined and stays declined.
Keep any binding rename work (V4) on its own branch.
