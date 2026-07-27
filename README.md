# Vicia DB

[![Build Status](https://github.com/etc-sw/vicia-db/actions/workflows/rust.yml/badge.svg)](https://github.com/etc-sw/vicia-db/actions/workflows/rust.yml)
[![Clippy Status](https://github.com/etc-sw/vicia-db/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/etc-sw/vicia-db/actions/workflows/rust-clippy.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/etc-sw/vicia-db#license)
[![Rust Edition](https://img.shields.io/badge/rust-2024-orange.svg)](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html)

> ### This is a fork
>
> ⚠️ This repository is **Vicia DB**, a fork of
> [project-minigraf/minigraf](https://github.com/project-minigraf/minigraf)
> maintained independently for the Vetch line. It is **not** the upstream
> project and is not endorsed by it.
>
> - **Nothing here is published.** This fork publishes no crate, npm package,
>   or release. Every package named `minigraf` below — on crates.io, npm, PyPI,
>   and Maven Central — is **upstream's**, and reflects upstream's code, not
>   this fork's.
> - **The badges above are this fork's own CI.** Upstream's build status,
>   coverage, and releases are on the upstream repository.
> - Work here has diverged from upstream: v10–v13 delta storage, bounded
>   transaction-pinned read views, copy-on-write recompact, and a browser
>   ledger split. See [CHANGELOG.md](CHANGELOG.md).
>
> Licensed under the original `MIT OR Apache-2.0` terms with upstream copyright
> notices preserved. See [License](#license).

> **Embedded graph memory for AI agents, mobile apps, and the browser** — the SQLite of bi-temporal graph databases

A tiny, self-contained graph database with **Datalog queries** and **bi-temporal time travel**. Think SQLite, but for connected data with full history.

## Vicia DB Transition

This repository is **Vicia DB**, the Vetch-oriented successor name for the
forked Minigraf line.

The Rust package rename has landed. The crate is `vicia-db`, the import path is
`vicia_db`, and `ViciaDb` is the primary handle type. `Minigraf` remains as a
type alias — it names the same type with the same API, so pre-rename source
keeps compiling — and is not deprecated. The version restarts at `0.1.0` rather
than continuing upstream's `1.1.1`; that number belongs to a different package
under a different maintainer.

**Publishing is deferred.** `vicia-db` is not on crates.io and
`@vicia-db/browser` is not on npm, by decision rather than by backlog. Every
consumer reaches this repository directly — by path, by pinned git revision, or
through a vendored package build — so a registry release would buy a docs.rs
page and a reserved name, at the cost of an irreversible publish and a release
pipeline that has to stay armed. That trade is worth making when there is an
outside consumer; there is not one yet. If you need one, open an issue.

The language bindings still carry upstream's names. They move in a later slice.

The file format does not change. `.graph` files, the `MGRF` header magic, and
every format version remain exactly as they are — format stability outranks
name consistency.

See [docs/VICIA_DB_RENAME_PLAN.md](docs/VICIA_DB_RENAME_PLAN.md) for the staged
rename plan, compatibility policy, downstream impact list, and attribution
checklist.

See [docs/MAINTENANCE_API_CONTRACT.md](docs/MAINTENANCE_API_CONTRACT.md) for
the `run_idle_maintenance()` caller contract.

## Local Vetch browser sync

Vicia's browser binding lives in `bindings/browser` and builds under the
package name `@vicia-db/browser`. Nothing is published to npm — the name is the
local package identity Vetch links against. Build the current checkout and
atomically sync it into the sibling Vetch checkout with:

```bash
just sync
```

To target a Vetch worktree explicitly:

```bash
just sync /absolute/path/to/vetch-worktree
```

The synced package includes `vicia-build.json` with the exact source commit,
dirty-state flag, wasm hash, and wasm-pack version. Vetch consumes it through a
`link:` dependency, so replacing the package directory takes effect without
overwriting `node_modules` or changing imports. This is the whole distribution
path today; if npm publication is ever taken up, it would replace Vetch's
dependency locator and nothing else.

## Vision

Vicia DB is a **single-file embedded graph database** that lets you:
- ✅ **Query relationships with Datalog** - Recursive rules, natural graph traversal
- ✅ **Time travel through history** - Bi-temporal queries (transaction time + valid time)
- ✅ **Forget without erasing history** - Atomically close valid-time windows for query results or fact lists
- ✅ **Window functions** - `sum/count/min/max/avg/rank/row-number :over (partition-by … :order-by …)` in `:find` clauses
- ✅ **Prepared statements** - Parse + plan once with `$slot` bind tokens, execute thousands of times
- ✅ **Embed anywhere** - Native, WASM, mobile, IoT - one `.graph` file
- ✅ **Zero configuration** - Just `ViciaDb::open("data.graph")` and you're done

**Status**: See [ROADMAP.md](ROADMAP.md) for current phase and what's next.

## Why Datalog?

**Datalog is fundamentally better for graphs than SQL-like languages:**

1. **Recursive by design** - Multi-hop traversals are natural, not an afterthought
2. **Simpler to implement** - Smaller spec = more reliable, faster to production
3. **Perfect for temporal** - Time is just another dimension in relations
4. **Proven at scale** - 40+ years of research, production use (Datomic, XTDB)
5. **Graph-native** - Facts (Entity-Attribute-Value) are literally edges
6. **LLM-friendly** - The small, uniform grammar (`[?e :attr ?v]` patterns, no JOIN variants, no subquery nesting) is easy for AI coding assistants to generate correctly from a few examples; the entire language fits in a system prompt

## Installation

`vicia-db` is **not on crates.io**, by decision — see
[Vicia DB Transition](#vicia-db-transition). Depend on this repository by path
or git revision:

```toml
[dependencies]
vicia-db = { git = "https://github.com/etc-sw/vicia-db" }
```

Then import through `vicia_db`:

```rust
use vicia_db::ViciaDb;
```

> **Note:** `cargo add minigraf` installs **upstream's** crate, not this fork.
> The two have diverged since v10. See [the fork notice](#this-is-a-fork).

## Quick Start

Use capability-scoped handles for ordinary application work. The interactive
handle can write and create bounded, transaction-pinned read views, but cannot
run maintenance, backup, or full export. Open the maintenance handle only in an
idle lifetime after the interactive handle has been dropped. Interactive writes
never hide threshold or drop checkpoint work; publication occurs only when the
maintenance lifetime explicitly requests it. Read-view row bounds stop source
and result generation at the first excess entry and reject the incomplete query
without truncation.

For an append-only history larger than 10,000 rows, create an `AnyValidTime`
read view and page one exact entity/attribute range with
`ReadView::entity_attribute_history`. The browser equivalent is
`BrowserReadView.entityAttributeHistory()`. Both retain assertion, retraction,
transaction, and valid-time identity under the pinned transaction cursor.

```rust
use vicia_db::{InteractiveLedger, MaintenanceLedger, ReadViewOptions};

{
    let ledger = InteractiveLedger::open("myapp.graph")?;
    ledger.execute_write(r#"(transact [[:alice :person/name "Alice"]
                                       [:alice :friend :bob]
                                       [:bob :person/name "Bob"]])"#)?;

    let view = ledger.read_view(ReadViewOptions::default())?;
    let _result = view.query(
        r#"(query [:find ?name
                   :where [:alice :friend ?friend]
                          [?friend :person/name ?name]])"#,
        16,
    )?;
}

{
    let maintenance = MaintenanceLedger::open("myapp.graph")?;
    maintenance.run_idle_maintenance()?;
    let projection = maintenance
        .rebuild_current_projections(&[":person/name".to_owned()])?;
    assert_eq!(projection.attribute_count, 1);
    let backup = maintenance.backup_to("myapp-backup.graph")?;
    assert!(backup.tx_count >= 1);
}
# Ok::<(), anyhow::Error>(())
```

### Raw compatibility and advanced Datalog

`ViciaDb` remains the supported unrestricted surface for rule registration,
prepared queries, semantic bulk forget, REPL-style execution, and migrations
from existing callers. It remains supported throughout 1.x; the replacement-
first 2.0 conditions and exact migration table are documented in
[`docs/API_COMPATIBILITY_AND_MIGRATION.md`](docs/API_COMPATIBILITY_AND_MIGRATION.md).

```rust
use vicia_db::{Vicia DB, OpenOptions};

// Open or create a file-backed database
let db = OpenOptions::new().path("myapp.graph").open()?;

// Add facts
db.execute(r#"(transact [[:alice :person/name "Alice"]
                         [:alice :person/age 30]
                         [:alice :friend :bob]
                         [:bob :person/name "Bob"]])"#)?;

// Query with Datalog
let results = db.execute(r#"
    (query [:find ?friend-name
            :where [:alice :friend ?friend]
                   [?friend :person/name ?friend-name]])
"#)?;

// Explicit transaction — all-or-nothing
let mut tx = db.begin_write()?;
tx.execute(r#"(transact [[:alice :person/age 31]])"#)?;
tx.commit()?;

// Optional file-backed maintenance hook for idle/startup/shutdown windows.
// It checkpoints pending writes, then runs private delta maintenance if needed.
let _maintenance = db.run_idle_maintenance()?;

// Create an independent rollback point while the live writer remains open.
// The receipt names the exact transaction watermark contained in the backup.
let backup = db.backup_to("myapp-before-experiment.graph")?;
assert!(backup.tx_count >= 1);

// Time travel — query as of past transaction counter
db.execute("(query [:find ?age :as-of 1 :where [:alice :person/age ?age]])")?;

// Semantic forget — close matching valid-time windows in one transaction.
// Earlier :as-of snapshots remain unchanged.
db.execute(r#"(forget [:find ?e ?a ?v
                       :where [?e :person/inactive true]
                              [?e ?a ?v]])"#)?;

// Vicia-facing Rust code can use the compatibility alias for the same handle
let _vicia = vicia_db::ViciaDb::in_memory()?;

// Recursive rule — transitive reachability
db.execute(r#"(rule [(reachable ?a ?b) [?a :friend ?b]])
              (rule [(reachable ?a ?b) [?a :friend ?m] (reachable ?m ?b)])"#)?;

// Prepared statement — parse + plan once, execute many times
use vicia_db::BindValue;
let pq = db.prepare("(query [:find ?name :as-of $tx :where [$entity :person/name ?name]])")?;
let r1 = pq.execute(&[("tx", BindValue::TxCount(1)), ("entity", BindValue::Entity(alice_id))])?;
let r2 = pq.execute(&[("tx", BindValue::TxCount(2)), ("entity", BindValue::Entity(bob_id))])?;
```

```bash
cargo run          # interactive Datalog REPL
cargo test         # run the native test suite
cargo run < demos/demo_recursive.txt   # recursive rules demo
```

## Demo

See a working implementation of **temporal reasoning** with Vicia DB at [github.com/adityamukho/temporal_reasoning](https://github.com/adityamukho/temporal_reasoning) — an AI agent that uses Vicia DB's bi-temporal model to store, correct, and audit beliefs.

See the [Datalog Reference](https://github.com/project-minigraf/minigraf/wiki/Datalog-Reference) wiki page for the complete syntax.

## Why Vicia DB?

No other database offers this combination:

| Feature | Vicia DB | XTDB | Cozo | Neo4j | SQLite |
|---|---|---|---|---|---|
| **Query Language** | Datalog | Datalog | Datalog | Cypher | SQL |
| **Single File** | ✅ Yes | ❌ No | ❌ No | ❌ No | ✅ Yes |
| **Bi-temporal** | ✅ Yes | ✅ Yes | ⚠️ Time travel | ❌ No | ❌ No |
| **Embedded** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| **Graph Native** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |
| **Rust** | ✅ Yes | ❌ Clojure | ✅ Yes | ❌ Java | ❌ C |
| **WASM Ready** | ✅ Yes (browser + WASI + 6 targets) | ❌ No | ⚠️ Limited | ❌ No | ✅ Yes |

## Platform support

> All packages listed here are published by **upstream Vicia DB**, not by this
> fork. They track upstream's code. The one exception is the browser binding in
> `bindings/browser`, which this fork builds locally as `@vicia-db/browser` and
> does not publish either.

| Platform | Package | Install |
|---|---|---|
| Rust (native) | `minigraf` on crates.io | `cargo add minigraf` |
| Browser WASM | `@minigraf/browser` on npm | `npm install @minigraf/browser` |
| WASI | `@minigraf/wasi` on npm, `.wasm` on GitHub Releases | `npm install @minigraf/wasi` |
| Node.js | `minigraf` on npm | `npm install minigraf` |
| Python | `minigraf` on PyPI | `pip install minigraf` |
| Java/JVM | `io.github.adityamukho:minigraf-jvm` on Maven Central | see [wiki](https://github.com/project-minigraf/minigraf/wiki/Use-Cases) |
| Android | `.aar` on GitHub Packages | see [wiki](https://github.com/project-minigraf/minigraf/wiki/Use-Cases) |
| iOS / macOS | `.xcframework` via Swift Package Manager | see [wiki](https://github.com/project-minigraf/minigraf/wiki/Use-Cases) |
| C / FFI | header + tarball on GitHub Releases | see [wiki](https://github.com/project-minigraf/minigraf/wiki/Use-Cases) |

**Embedded graph memory for agents, mobile, and the browser — SQLite's simplicity + Datomic's temporal model.**

## Language Bindings

| Language | Package | Repo |
|---|---|---|
| Python | [`minigraf` on PyPI](https://pypi.org/p/minigraf) | [minigraf-python](https://github.com/project-minigraf/minigraf-python) |
| Node.js | [`minigraf` on npm](https://www.npmjs.com/package/minigraf) | [minigraf-node](https://github.com/project-minigraf/minigraf-node) |
| Browser WASM | [`@minigraf/browser` on npm](https://www.npmjs.com/package/@minigraf/browser) | [minigraf-wasm](https://github.com/project-minigraf/minigraf-wasm) |
| WASI | [`@minigraf/wasi` on npm](https://www.npmjs.com/package/@minigraf/wasi) | [minigraf-wasm](https://github.com/project-minigraf/minigraf-wasm) |
| Java | (in this repo — Phase 2 split pending) | — |
| Android | (in this repo — Phase 2 split pending) | — |
| iOS/macOS | (in this repo — Phase 2 split pending) | — |
| C | [`minigraf-c`](./minigraf-c) (in this repo) | — |

See the [Comparison](https://github.com/project-minigraf/minigraf/wiki/Comparison) wiki page for detailed analysis including temporal vs. time-series databases.

### For AI Agents

Store what an agent believes, retract and correct without losing history, and replay past states to audit decisions. Every fact carries both transaction time (when it was recorded) and valid time (when it was true), so you can reconstruct the exact knowledge state at the moment of any past decision.

Pairs well with vector stores (GraphRAG pattern): the vector store answers "what is similar?"; Vicia DB answers "what are the relationships, who recorded them, and what did we believe at time T?"

### For Mobile Apps

Offline-first storage with retroactive corrections — the bi-temporal model lets you correct a mis-entered value while preserving the original record. Native Kotlin and Swift bindings ship as an Android `.aar` (GitHub Packages) and an iOS `.xcframework` (Swift Package Manager) via [UniFFI](https://github.com/mozilla/uniffi-rs). No Rust required.

```kotlin
// Android (Kotlin)
val db = MiniGrafDb.open(context.filesDir.absolutePath + "/myapp.graph")
db.execute("""(transact [[:alice :person/name "Alice"] [:alice :person/age 30]])""")
val json = db.execute("(query [:find ?name :where [?e :person/name ?name]])")
```

```swift
// iOS (Swift)
let db = try MiniGrafDb.open(path: docsURL.appendingPathComponent("myapp.graph").path)
try db.execute(datalog: #"(transact [[:alice :person/name "Alice"] [:alice :person/age 30]])"#)
let json = try db.execute(datalog: "(query [:find ?name :where [?e :person/name ?name]])")
```

See the [Mobile Integration](https://github.com/project-minigraf/minigraf/wiki/Use-Cases#mobile-apps) wiki section for full setup and usage docs (Gradle config, SPM integration, error handling, threading).

### For WASM / Browser

Published compatibility package: [`@minigraf/browser`](https://www.npmjs.com/package/@minigraf/browser) (IndexedDB-backed, `wasm-pack`). Vetch consumes the current checkout through its local `@vicia-db/browser` package boundary. `BrowserDb` remains the low-level 1.x compatibility surface. Ordinary foreground callers should open `BrowserInteractiveLedger`, which always uses the paged path and exposes only bounded transaction-pinned read views plus `executeAtomic(commands)`. Reads require row and byte budgets, reject unindexed plans or incomplete results, and select current, any-valid-time, `asOf`, or exact valid time through the read-view constructor. Portability and O(total) work belong to `BrowserMaintenanceLedger`, which owns verified export, strict import, caller-scheduled idle maintenance, and explicit current-projection rebuilds but cannot query or write. Keep foreground writer ownership under a Web Lock; run legacy migration, import, full export, and maintenance in a disposable worker that acquires the same lock, reports its outcome, terminates, and lets the caller reopen the interactive capability. See the runnable [`examples/browser`](examples/browser) flow, [`docs/API_COMPATIBILITY_AND_MIGRATION.md`](docs/API_COMPATIBILITY_AND_MIGRATION.md), [`docs/DURABILITY_AND_CALLER_RULES.md`](docs/DURABILITY_AND_CALLER_RULES.md), and [`docs/MAINTENANCE_API_CONTRACT.md`](docs/MAINTENANCE_API_CONTRACT.md).

WASI build (`wasm32-wasip1`) remains available as [`@minigraf/wasi`](https://www.npmjs.com/package/@minigraf/wasi) and as a GitHub Releases artifact. File format v12 retains v11's generation-bound, page-local base integrity checks and adds adaptive prefix-compressed index leaves. V11 remains directly readable: foreground open and delta checkpoints preserve its bytes, while caller-scheduled idle maintenance is the COW upgrade boundary. Explicit native and browser maintenance capabilities can publish a bounded attribute list as a v13 current-projection catalog; ordinary writes remain v12. Transaction-pinned read views use the catalog only for exact-watermark, single-attribute, ungrouped `count`/`sum` queries, with a bounded resident tail overlay keeping post-publication writes on that route and generation changes, unsupported query shapes, corrupt projection copies, or over-budget tails falling back to the full-history ledger. Browser publication is one atomic IndexedDB transaction and reopens the selected v13 authority after commit. On the recorded 1M-fact Chrome 150 matrix, paged open starts in a 17.8 ms five-run maximum and the open-plus-six-probe phase adds at most 51.1 MiB of sampled PSS; one-fact writes remain 8.3 ms p95. Legacy migration, import, full verified export, recompact, and projection rebuild are explicit O(total) operations and must run in a disposable worker. R2-C5 and its bounded-contract correction deliver the complete v13 boundary to Vetch from clean source `77a3008`; the generated JS glue, declarations, manifests, licenses, WASM, and provenance pass the 78-test real-Chrome package matrix, complete Vetch authority suite, TypeScript check, and production build. Foreground read views independently cap complete results at 10,000 rows, while resident projection tails count empty replacements toward both their entity and byte bounds. The package version, ordinary v12 writer default, and Vetch scheduling policy remain unchanged. R3 bounded-memory recompact is next.

### For Python / Node.js / Java / C

Language bindings ship as `minigraf` on PyPI, `minigraf` on npm (Node.js native addon), `io.github.adityamukho:minigraf-jvm` on Maven Central, and a C header + prebuilt shared library on GitHub Releases. See the [Use Cases wiki](https://github.com/project-minigraf/minigraf/wiki/Use-Cases).

## Scope

Vicia DB runs as:
- ✅ An embedded library
- ✅ A standalone binary (interactive REPL)
- ✅ Browser WASM — `@minigraf/browser` (IndexedDB-backed, `wasm-pack`)
- ✅ Server-side WASM — `wasm32-wasip1` / WASI (Wasmtime, Wasmer, Cloudflare Workers)
- ✅ Android, iOS, Python, Node.js, Java, C — via UniFFI / napi-rs / cbindgen

Vicia DB will **not** be (by design):
- **Distributed** — no clustering, no sharding, no replication; each agent instance owns its own `.graph` file
- **Client-server** — no network protocol in core
- **Billion-node scale** — optimised for <1M nodes (like SQLite)
- **A time-series database** — Vicia DB is a *temporal* database; see [Comparison](https://github.com/project-minigraf/minigraf/wiki/Comparison#influxdb--prometheus--timescaledb-time-series-databases)

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full phase plan, current status, and release strategy.

## Performance

These numbers come from **two different machines**; cross-machine comparison is
indicative only. See [BENCHMARKS.md](docs/BENCHMARKS.md) for full tables and
per-row provenance, [BENCHMARK_MILESTONES.md](docs/BENCHMARK_MILESTONES.md) for
the machine-checked development and release gates, and
[CROSS_DB_STRESS_BENCHMARK.md](docs/CROSS_DB_STRESS_BENCHMARK.md) for the
classified Vicia/CozoDB/SQLite/redb speed, memory, storage, reopen, and kill-9
comparison.

- **H0** — Intel Core i7-1065G7 @ 1.30GHz, 16 GB, Rust 1.94.0
- **A0** — AMD Ryzen 7 7800X3D, 32 GB, WSL2, Rust 1.96.0-nightly

| Metric | Result | Host |
|---|---|---|
| Insert (in-memory, single fact) | ~2.7 µs — flat across 1K–100K facts | H0 |
| Insert (file-backed, WAL) | ~3.6 µs — flat across 1K–100K facts | H0 |
| Entity+attribute point query at 1M facts | 4.1 µs (selective B+tree lookup) | A0 |
| Entity-bound `:as-of` point read at 1M facts | 0.017–0.043 ms p95 (Q1-B selective pushdown; was ~1.26–1.50 s) | H0 |
| Attribute-wide query at 1M facts | 485 ms (result set grows with N) | A0 |
| Delta checkpoint at 1M facts, one pending fact | 512 ms (was 4.83 s on full rebuild) | H0 |
| Checkpoint p95, 1M base × 1,024 receipt slices | 3.098 ms (exact caller trace, budget 50 ms) | A0 |
| Reopen after delta publish at 1M facts | 4.4 ms (same trace) | A0 |
| Open time at 1M facts (v6-era, cold committed file) | 1.31 s | H0 |
| Peak heap at 1M facts (v6-era) | 1.05 GB | H0 |

The last two rows date from the v6 format and have not been re-measured since;
the current format is v12 with an explicit v13 current-projection catalog.

File-backed databases enforce a maximum fact size of **4 080 serialised bytes** per fact. In-memory databases have no limit.

## Contributing

This is a hobby project with a long-term vision. Read [PHILOSOPHY.md](PHILOSOPHY.md) and [ROADMAP.md](ROADMAP.md) before proposing features.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code standards, and the PR process.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

Vicia DB successor work preserves the Vicia DB lineage, original copyright
notice, and dual-license terms unless a future legal review explicitly changes
that policy.
