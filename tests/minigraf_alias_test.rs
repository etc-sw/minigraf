//! The `Minigraf` compatibility alias must keep naming the primary handle.
//!
//! `ViciaDb` is the primary type since the package rename. These tests exist so
//! that pre-rename source — which spells the handle `Minigraf` — keeps
//! compiling and keeps reading the same on-disk format. They deliberately mix
//! both spellings in one expression so a future accidental split into two
//! distinct types fails to compile here first.

use vicia_db::{Minigraf, QueryResult, Value, ViciaDb};

fn assert_single_string(result: QueryResult, expected: &str) {
    match result {
        QueryResult::QueryResults { results, .. } => {
            assert_eq!(results.len(), 1, "expected one query row");
            let row = results.first().expect("expected query row");
            let value = row.first().expect("expected first query column");
            assert_eq!(
                value,
                &Value::String(expected.to_owned()),
                "expected query value"
            );
        }
        _ => panic!("expected query results"),
    }
}

#[test]
fn minigraf_alias_uses_the_vicia_db_api_in_memory() {
    let db = Minigraf::in_memory().expect("in-memory db should open");
    let primary: ViciaDb = db.clone();

    db.execute(r#"(transact [[:alice :person/name "Alice"]])"#)
        .expect("transact should succeed");

    let result = primary
        .execute(r#"(query [:find ?name :where [:alice :person/name ?name]])"#)
        .expect("query should succeed");

    assert_single_string(result, "Alice");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_minigraf_written_file_reopens_through_vicia_db() {
    let dir = tempfile::tempdir().expect("tempdir should create");
    let path = dir.path().join("minigraf-alias.graph");

    {
        let db = Minigraf::open(&path).expect("file db should open");
        db.execute(r#"(transact [[:bob :person/name "Bob"]])"#)
            .expect("transact should succeed");
        db.checkpoint().expect("checkpoint should succeed");
    }

    let reopened = ViciaDb::open(&path).expect("file db should reopen");
    let result = reopened
        .execute(r#"(query [:find ?name :where [:bob :person/name ?name]])"#)
        .expect("query should succeed");

    assert_single_string(result, "Bob");
}
