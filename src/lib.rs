#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
    )
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Zero-config, single-file, embedded graph database with bi-temporal Datalog queries.
//!
//! Vicia DB is the SQLite of graph databases: embedded, no server, no configuration,
//! a single portable `.graph` file. It stores data as Entity-Attribute-Value facts,
//! queries them with [Datalog](https://en.wikipedia.org/wiki/Datalog), and tracks every
//! change with full bi-temporal history (transaction time + valid time).
//!
//! # Installation
//!
//! ```toml
//! [dependencies]
//! vicia-db = "0.1"
//! ```
//!
//! # Quick Start
//!
//! ```
//! use vicia_db::{ViciaDb, BindValue};
//!
//! // Open (or create) a database
//! let db = ViciaDb::in_memory().unwrap();
//!
//! // Assert facts
//! db.execute(r#"(transact [[:alice :person/name "Alice"]
//!                          [:alice :person/age 30]
//!                          [:alice :friend :bob]
//!                          [:bob   :person/name "Bob"]])"#).unwrap();
//!
//! // Query with Datalog
//! let results = db.execute(r#"
//!     (query [:find ?friend-name
//!             :where [:alice :friend ?friend]
//!                    [?friend :person/name ?friend-name]])
//! "#).unwrap();
//!
//! // Explicit transaction — all-or-nothing
//! let mut tx = db.begin_write().unwrap();
//! tx.execute(r#"(transact [[:alice :person/age 31]])"#).unwrap();
//! tx.commit().unwrap();
//!
//! // Time travel — query the state as of transaction 1
//! db.execute("(query [:find ?age :as-of 1 :where [:alice :person/age ?age]])").unwrap();
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Target | Description |
//! |---------|--------|-------------|
//! | *(default)* | native / WASI | File-backed and in-memory databases; full Datalog engine |
//! | `browser` | `wasm32-unknown-unknown` | Enables the `browser` module (`BrowserDb`), backed by IndexedDB for use inside a web browser via `wasm-pack` |
//! | `bench-internals` | native | Exposes repository-only cursor and pending/WAL memory diagnostics for benchmark receipts; not a stable application API |
//!
//! The `browser` feature is only meaningful on the `wasm32-unknown-unknown` target.
//! When browsing docs on [docs.rs](https://docs.rs/vicia-db), switch the target to
//! `wasm32-unknown-unknown` (top-right target selector) to see the full browser API.
//!
//! ## WebAssembly targets
//!
//! - **Browser** (`wasm32-unknown-unknown` + `browser` feature) — `wasm-pack build --target web --features browser`
//! - **WASI / server-side** (`wasm32-wasip1`) — `cargo build --target wasm32-wasip1 --release --bin vicia-db`

pub mod db;
#[cfg(test)]
pub(crate) mod gate_e_test_support;
pub(crate) mod graph;
pub(crate) mod json_value;
pub(crate) mod query;
/// Interactive REPL for exploring a [`ViciaDb`] database from the command line.
pub mod repl;
/// A6 framed pipe session mode — NDJSON protocol for caller-owned child processes.
#[cfg(not(target_arch = "wasm32"))]
pub mod session;
pub(crate) mod storage;
pub(crate) mod temporal;
pub(crate) mod wal;

#[cfg(all(target_arch = "wasm32", feature = "browser"))]
#[cfg_attr(docsrs, doc(cfg(all(target_arch = "wasm32", feature = "browser"))))]
pub mod browser;

#[cfg(not(target_arch = "wasm32"))]
pub use db::{BackupOutcome, OpenOptionsWithPath};
pub use db::{
    CURRENT_ENTITIES_MAX_ATTRIBUTES, CURRENT_ENTITIES_MAX_HISTORY_ENTRIES,
    CURRENT_ENTITIES_MAX_IDS, CURRENT_ENTITIES_MAX_PAIRS, CURRENT_PROJECTION_MAX_ATTRIBUTES,
    CURRENT_REFS_MAX_HISTORY_ENTRIES, CurrentEntitiesRequest, CurrentFact, CurrentRefsRequest,
    ENTITY_ATTRIBUTE_HISTORY_MAX_RESULT_BYTES, ENTITY_ATTRIBUTE_HISTORY_MAX_SOURCE_ENTRIES,
    EntityAttributeHistoryCursor, EntityAttributeHistoryPage, EntityAttributeHistoryRequest,
    InteractiveLedger, InteractiveWriteTransaction, MaintenanceAdvice, MaintenanceCheckpointEffect,
    MaintenanceDeltaEffect, MaintenanceLedger, MaintenanceOutcome, OpenOptions,
    ProjectionMaintenanceOutcome, READ_VIEW_MAX_QUERY_WORK_ROWS, READ_VIEW_MAX_ROWS, ReadView,
    ReadViewOptions, ReadViewValidAt, VALID_TIME_DIFF_MAX_IDS, VALID_TIME_DIFF_MAX_RESULT_BYTES,
    VALID_TIME_DIFF_MAX_SOURCE_ENTRIES, ValidTimeDiffChange, ValidTimeDiffCursor,
    ValidTimeDiffPage, ValidTimeDiffRequest, ValidTimeDiffRow, ViciaDb, WriteTransaction,
};
/// Pre-rename name for the primary embedded database handle, kept as an alias.
///
/// [`ViciaDb`] is the primary type. This alias exists so code written against
/// the `minigraf` package keeps compiling after the rename — it names the same
/// type, the same API, and the same on-disk format. It is not deprecated; the
/// fork simply no longer leads with the upstream name.
pub type Minigraf = ViciaDb;
pub use repl::Repl;

// EAV value types — users construct and match on these
pub use graph::types::{EntityId, FactRecord, FactValidTime, Value};

#[cfg(feature = "bench-internals")]
pub use db::{AtomicWritePreparationDiagnostics, WalReplayMemoryDiagnostics};
#[cfg(any(test, feature = "bench-internals"))]
pub use graph::current_projection::{
    CurrentProjectionCandidate, CurrentProjectionRefreshDiagnostics,
};
#[cfg(feature = "bench-internals")]
pub use graph::storage::{
    CurrentAttributeCursorDiagnostics, PendingMemoryComponent, PendingMemoryDiagnostics,
};
#[cfg(feature = "bench-internals")]
pub use storage::btree_v6::LeafReadDiagnostics;
#[cfg(any(test, feature = "bench-internals"))]
pub use storage::current_projection_image::{
    CurrentProjectionPageImage, ProjectionReadDiagnostics,
};
#[cfg(feature = "bench-internals")]
pub use storage::layout_diagnostics::{
    PrefixEstimate, StorageIndexLayout, StorageLayoutDiagnostics, StoragePageLayout,
    inspect_storage_layout,
};
#[cfg(feature = "bench-internals")]
pub use storage::persistent_facts::CheckpointConstructionDiagnostics;
#[cfg(any(test, feature = "bench-internals"))]
pub use storage::persistent_facts::ProjectionPublicationReceipt;
#[cfg(any(test, feature = "bench-internals"))]
pub use storage::projection_catalog::ProjectionLedgerIdentity;

// Query result type
pub use query::datalog::executor::QueryResult;

// Bi-temporal query types
pub use query::datalog::types::{AsOf, ValidAt};

// Prepared statements
pub use query::datalog::prepared::{BindValue, PreparedQuery};
