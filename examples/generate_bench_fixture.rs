//! Generates a checkpointed `.graph` fixture of N facts for the A0
//! browser open-at-scale runner (docs/internal/APP_ADOPTION_GAP_PLAN.md).
//!
//!   cargo run --release --example generate_bench_fixture -- <facts> <out.graph> [diff-entities]
//!
//! Fact shape matches the delta/cadence benchmark base (`:bench/base-{i}`
//! cycling ref/value/keyword/flag) so browser numbers are comparable with
//! the native suites. Output is fully checkpointed with no WAL sidecar.
//!
//! The optional third argument appends `diff-entities` valid-time receipt
//! entities on top of the base, each carrying one replaced `:status/value`
//! window (`:old` valid 2020-01-01..2021-01-01, `:new` valid 2021-01-01..
//! forever). A diff between those two years therefore yields exactly one
//! `Disappeared` plus one `Appeared` row per entity. Entity ids, attribute,
//! and instants match `measure_valid_time_diff_1m` in
//! `tests/valid_time_diff_test.rs`, so the native A-2 gate and a browser run
//! can be pointed at the same fixture with the same request. Omitting the
//! argument leaves the base-only fact set unchanged.
//!
//! Output is not byte-reproducible in either mode: `tx_id` is a wall-clock
//! timestamp, so two runs of the same command differ. Receipts pin the
//! SHA256 of the one generated file they measured, never a rebuilt one.

// wasm-pack compiles examples for the browser target; provide a no-op entry
// point so the example compiles cleanly.
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    use uuid::Uuid;

    let mut args = std::env::args().skip(1);
    let facts: usize = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: generate_bench_fixture <facts> <out.graph>"))?
        .parse()?;
    let out = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: generate_bench_fixture <facts> <out.graph>"))?;
    let diff_entities: usize = match args.next() {
        Some(argument) => argument.parse()?,
        None => 0,
    };

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(format!("{out}.wal"));

    let db = minigraf::OpenOptions {
        wal_checkpoint_threshold: usize::MAX,
        ..Default::default()
    }
    .path(&out)
    .open()?;

    const BATCH: usize = 1_000;
    for batch_start in (0..facts).step_by(BATCH) {
        let batch_end = (batch_start + BATCH).min(facts);
        let mut command = String::from("(transact [");
        for index in batch_start..batch_end {
            let entity = format!(":bench/base-{index}");
            if index % 4 == 0 {
                let target = Uuid::from_u128(index as u128 + 1);
                command.push_str(&format!(r#"[{entity} :bench/ref #uuid "{target}"]"#));
            } else if index % 4 == 1 {
                command.push_str(&format!("[{entity} :bench/value {index}]"));
            } else if index % 4 == 2 {
                command.push_str(&format!("[{entity} :bench/state :bench/state-{index}]"));
            } else {
                command.push_str(&format!("[{entity} :bench/flag true]"));
            }
        }
        command.push_str("])");
        db.execute(&command)?;
    }
    // Valid-time receipt entities. One `(transact ...)` per window, matching
    // `measure_valid_time_diff_1m` so the same request returns the same rows
    // against either fixture.
    const RECEIPT_ENTITY_BASE: u128 = 0x9000_0000;
    for index in 0..diff_entities {
        let entity = Uuid::from_u128(RECEIPT_ENTITY_BASE + index as u128);
        db.execute(&format!(
            r#"(transact {{:valid-from "2020-01-01" :valid-to "2021-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
        ))?;
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :status/value :new]])"#
        ))?;
    }

    db.checkpoint()?;
    drop(db);

    let _ = std::fs::remove_file(format!("{out}.wal"));
    let len = std::fs::metadata(&out)?.len();
    let total = facts + diff_entities * 2;
    println!(
        "Written: {out} ({total} facts = {facts} base + {} valid-time window, {len} bytes)",
        diff_entities * 2
    );
    Ok(())
}
