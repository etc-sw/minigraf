use minigraf::{
    EntityAttributeHistoryRequest, FactValidTime, Minigraf, ReadViewOptions, ReadViewValidAt, Value,
};

fn history_view(db: &Minigraf) -> minigraf::ReadView<'_> {
    db.read_view(ReadViewOptions {
        as_of: None,
        valid_at: ReadViewValidAt::AnyValidTime,
    })
    .expect("history view")
}

#[test]
fn exact_history_pages_more_than_ten_thousand_rows_without_loss() {
    let entity = uuid::Uuid::from_u128(1);
    let mut facts = String::new();
    for value in 1u128..=12_005 {
        facts.push_str(&format!(
            r#"[#uuid "{entity}" :canvas/root-link #uuid "{}"] "#,
            uuid::Uuid::from_u128(value)
        ));
    }
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!("(transact [{facts}])"))
        .expect("root-link transaction");
    let view = history_view(&db);

    db.execute(&format!(
        r#"(transact [[#uuid "{entity}" :canvas/root-link #uuid "{}"]])"#,
        uuid::Uuid::from_u128(20_000)
    ))
    .expect("post-view transaction");

    let first = view
        .entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":canvas/root-link",
            after: None,
            limit: 10_000,
        })
        .expect("first history page");
    assert_eq!(first.facts.len(), 10_000);
    let continuation = first.next.expect("first page continuation");
    let second = view
        .entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":canvas/root-link",
            after: Some(&continuation),
            limit: 10_000,
        })
        .expect("second history page");
    assert_eq!(second.facts.len(), 2_005);
    assert!(second.next.is_none());

    let values = first
        .facts
        .into_iter()
        .chain(second.facts)
        .map(|fact| fact.value)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 12_005);
    for (offset, value) in values.into_iter().enumerate() {
        assert_eq!(value, Value::Ref(uuid::Uuid::from_u128(offset as u128 + 1)));
    }
}

#[test]
fn exact_history_preserves_scoped_and_unscoped_retractions() {
    let entity = uuid::Uuid::from_u128(2);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-01-01" :valid-to "2021-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("first assertion");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01" :valid-to "2022-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("second assertion");
    db.execute(&format!(
        r#"(retract {{:valid-from "2020-01-01" :valid-to "2021-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("scoped retraction");
    db.execute(&format!(
        r#"(retract [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("unscoped retraction");

    let page = history_view(&db)
        .entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":status/value",
            after: None,
            limit: 8,
        })
        .expect("complete history");
    assert_eq!(page.facts.len(), 4);
    assert!(page.next.is_none());
    assert_eq!(page.facts.iter().filter(|fact| fact.asserted).count(), 2);
    assert_eq!(page.facts.iter().filter(|fact| !fact.asserted).count(), 2);
    assert!(
        page.facts.iter().any(|fact| {
            !fact.asserted && matches!(fact.valid_time, FactValidTime::AllValidTime)
        })
    );
    assert!(page.facts.iter().any(|fact| {
        !fact.asserted
            && matches!(
                fact.valid_time,
                FactValidTime::Window {
                    valid_from: 1_577_836_800_000,
                    valid_to: 1_609_459_200_000,
                }
            )
    }));
}

#[test]
fn exact_history_merges_committed_and_pending_rows() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("entity-attribute-history.graph");
    let entity = uuid::Uuid::from_u128(3);
    {
        let db = Minigraf::open(&path).expect("open database");
        db.execute(&format!(
            r#"(transact [[#uuid "{entity}" :card/event "committed"] [#uuid "{entity}" :card/event 1.25]])"#
        ))
        .expect("committed transaction");
        db.checkpoint().expect("checkpoint");
    }

    let db = Minigraf::open(&path).expect("reopen database");
    db.execute(&format!(
        r#"(transact [[#uuid "{entity}" :card/event "pending"]])"#
    ))
    .expect("pending transaction");
    let page = history_view(&db)
        .entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":card/event",
            after: None,
            limit: 4,
        })
        .expect("merged history");
    assert_eq!(page.facts.len(), 3);
    assert!(
        page.facts
            .iter()
            .any(|fact| fact.value == Value::String("committed".to_owned()))
    );
    assert!(
        page.facts
            .iter()
            .any(|fact| fact.value == Value::String("pending".to_owned()))
    );
    assert!(
        page.facts
            .iter()
            .any(|fact| fact.value == Value::Float(1.25))
    );
}

#[test]
fn exact_history_rejects_wrong_view_range_cursor_and_limit() {
    let entity = uuid::Uuid::from_u128(4);
    let other = uuid::Uuid::from_u128(5);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact [[#uuid "{entity}" :card/event "one"] [#uuid "{entity}" :card/event "two"]])"#
    ))
    .expect("transaction");

    assert!(
        db.read_view(ReadViewOptions::default())
            .expect("current view")
            .entity_attribute_history(EntityAttributeHistoryRequest {
                entity,
                attribute: ":card/event",
                after: None,
                limit: 1,
            })
            .is_err()
    );
    let view = history_view(&db);
    let first = view
        .entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":card/event",
            after: None,
            limit: 1,
        })
        .expect("first page");
    let cursor = first.next.expect("continuation");
    assert!(
        view.entity_attribute_history(EntityAttributeHistoryRequest {
            entity: other,
            attribute: ":card/event",
            after: Some(&cursor),
            limit: 1,
        })
        .is_err()
    );
    assert!(
        view.entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":db/tx-count",
            after: None,
            limit: 1,
        })
        .is_err()
    );
    assert!(
        view.entity_attribute_history(EntityAttributeHistoryRequest {
            entity,
            attribute: ":card/event",
            after: None,
            limit: 10_001,
        })
        .is_err()
    );
}

#[test]
fn exact_history_rejects_pages_larger_than_eight_mib() {
    let entity = uuid::Uuid::from_u128(6);
    let payload = "v".repeat(1_700);
    let facts = (0..5_000)
        .map(|index| format!(r#"[#uuid "{entity}" :chunk/body "{index:04}-{payload}"]"#))
        .collect::<Vec<_>>()
        .join(" ");
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!("(transact [{facts}])"))
        .expect("large transaction");
    assert!(
        history_view(&db)
            .entity_attribute_history(EntityAttributeHistoryRequest {
                entity,
                attribute: ":chunk/body",
                after: None,
                limit: 5_000,
            })
            .is_err()
    );
}
