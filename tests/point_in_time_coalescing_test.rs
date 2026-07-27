use anyhow::{Result, bail};
use vicia_db::{QueryResult, Value, ViciaDb};

fn rows(db: &ViciaDb, query: &str) -> Result<Vec<Vec<Value>>> {
    match db.execute(query)? {
        QueryResult::QueryResults { results, .. } => Ok(results),
        _ => bail!("expected query results"),
    }
}

fn insert_overlapping_claim_windows(db: &ViciaDb) -> Result<()> {
    for (valid_from, valid_to) in [("2020-01-01", "2090-01-01"), ("2025-01-01", "2100-01-01")] {
        db.execute(&format!(
            r#"(transact {{:valid-from "{valid_from}" :valid-to "{valid_to}"}} [
                [:claim :lease/live true]
                [:claim :lease/expires-ms 4102444800000]
                [:claim :lease/owner :agent/vetch-codex]
            ])"#
        ))?;
    }
    Ok(())
}

fn assert_one_claim_row(db: &ViciaDb, valid_at: &str) -> Result<()> {
    let result = rows(
        db,
        &format!(
            r#"(query [:find ?live ?expires ?owner {valid_at}
                :where [:claim :lease/live ?live]
                       [:claim :lease/expires-ms ?expires]
                       [:claim :lease/owner ?owner]
                :max-results 6])"#
        ),
    )?;
    assert_eq!(
        result.len(),
        1,
        "overlapping windows must produce one logical joined row"
    );
    Ok(())
}

#[test]
fn explicit_and_default_point_in_time_relations_coalesce_before_join() -> Result<()> {
    let db = ViciaDb::in_memory()?;
    insert_overlapping_claim_windows(&db)?;

    assert_one_claim_row(&db, r#":valid-at "2026-07-17""#)?;
    assert_one_claim_row(&db, "")?;
    Ok(())
}

#[test]
fn any_valid_time_retains_distinct_overlapping_windows() -> Result<()> {
    let db = ViciaDb::in_memory()?;
    insert_overlapping_claim_windows(&db)?;

    let result = rows(
        &db,
        r#"(query [:find ?vf ?vt
            :any-valid-time
            :where [:claim :lease/live true]
                   [:claim :db/valid-from ?vf]
                   [:claim :db/valid-to ?vt]])"#,
    )?;
    assert_eq!(
        result.len(),
        2,
        "history query must retain both validity windows"
    );
    Ok(())
}
