use minigraf::{
    Minigraf, ReadViewOptions, ReadViewValidAt, ValidTimeDiffChange, ValidTimeDiffRequest, Value,
};

const T_2018_07: i64 = 1_530_403_200_000;
const T_2019_01: i64 = 1_546_300_800_000;
const T_2019_07: i64 = 1_561_939_200_000;
const T_2020_01: i64 = 1_577_836_800_000;
const T_2020_07: i64 = 1_593_561_600_000;
const T_2021_01: i64 = 1_609_459_200_000;
const T_2021_07: i64 = 1_625_097_600_000;
const T_2022_01: i64 = 1_640_995_200_000;
const T_2022_07: i64 = 1_656_633_600_000;

fn view_any(db: &Minigraf) -> minigraf::ReadView<'_> {
    db.read_view(ReadViewOptions {
        as_of: None,
        valid_at: ReadViewValidAt::AnyValidTime,
    })
    .expect("any-valid-time view")
}

fn run_diff(
    view: &minigraf::ReadView<'_>,
    attribute: &str,
    entities: Option<&[uuid::Uuid]>,
    before: i64,
    after: i64,
    limit: usize,
) -> minigraf::ValidTimeDiffPage {
    view.valid_time_diff(ValidTimeDiffRequest {
        attribute,
        entities,
        valid_at_before: before,
        valid_at_after: after,
        after: None,
        limit,
    })
    .expect("diff page")
}

#[test]
fn appeared_disappeared_and_untouched_values_between_two_instants() {
    let entity = uuid::Uuid::from_u128(1);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-01-01" :valid-to "2021-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("old assertion");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :status/value :new]])"#
    ))
    .expect("new assertion");
    db.execute(&format!(
        r#"(transact {{:valid-from "2019-01-01"}} [[#uuid "{entity}" :status/value :steady]])"#
    ))
    .expect("steady assertion");

    let page = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2020_07,
        T_2021_07,
        10,
    );
    assert_eq!(page.rows.len(), 2, "expected exactly two changed values");
    assert!(page.next.is_none());
    let disappeared = page
        .rows
        .iter()
        .find(|row| row.change == ValidTimeDiffChange::Disappeared)
        .expect("disappeared row");
    assert_eq!(disappeared.entity, entity);
    assert_eq!(disappeared.attribute, ":status/value");
    assert_eq!(disappeared.value, Value::Keyword(":old".to_owned()));
    assert_eq!(disappeared.valid_from, T_2020_01);
    assert_eq!(disappeared.valid_to, T_2021_01);
    let appeared = page
        .rows
        .iter()
        .find(|row| row.change == ValidTimeDiffChange::Appeared)
        .expect("appeared row");
    assert_eq!(appeared.value, Value::Keyword(":new".to_owned()));
    assert_eq!(appeared.valid_from, T_2021_01);
    assert_eq!(appeared.valid_to, i64::MAX);
}

#[test]
fn multi_valued_attribute_emits_only_changed_values() {
    let entity = uuid::Uuid::from_u128(2);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-01-01"}} [[#uuid "{entity}" :card/tag :one] [#uuid "{entity}" :card/tag :two]])"#
    ))
    .expect("stable tags");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :three]])"#
    ))
    .expect("late tag");

    let page = run_diff(
        &view_any(&db),
        ":card/tag",
        Some(&[entity]),
        T_2020_07,
        T_2021_07,
        10,
    );
    assert_eq!(page.rows.len(), 1, "only the late tag changed");
    assert_eq!(page.rows[0].change, ValidTimeDiffChange::Appeared);
    assert_eq!(page.rows[0].value, Value::Keyword(":three".to_owned()));
}

#[test]
fn scoped_and_unscoped_retractions_shape_the_diff() {
    let entity = uuid::Uuid::from_u128(3);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-01-01" :valid-to "2022-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("windowed assertion");
    db.execute(&format!(
        r#"(retract {{:valid-from "2020-01-01" :valid-to "2022-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("scoped retraction");
    db.execute(&format!(
        r#"(transact {{:valid-from "2022-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
    ))
    .expect("re-assertion");

    let page = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2021_07,
        T_2022_07,
        10,
    );
    assert_eq!(page.rows.len(), 1, "retracted window must not appear at t1");
    assert_eq!(page.rows[0].change, ValidTimeDiffChange::Appeared);
    assert_eq!(page.rows[0].valid_from, T_2022_01);

    let gone = uuid::Uuid::from_u128(4);
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-01-01"}} [[#uuid "{gone}" :status/value :gone]])"#
    ))
    .expect("assertion to retract");
    db.execute(&format!(
        r#"(retract [[#uuid "{gone}" :status/value :gone]])"#
    ))
    .expect("unscoped retraction");
    let empty = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[gone]),
        T_2020_07,
        T_2021_07,
        10,
    );
    assert_eq!(
        empty.rows.len(),
        0,
        "unscoped retraction hides both instants"
    );
}

#[test]
fn interval_internal_churn_is_invisible() {
    let entity = uuid::Uuid::from_u128(5);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2020-03-01" :valid-to "2020-09-01"}} [[#uuid "{entity}" :status/value :blip]])"#
    ))
    .expect("short-lived window");

    let page = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2020_01,
        T_2021_01,
        10,
    );
    assert_eq!(
        page.rows.len(),
        0,
        "net diff must not report interval-internal churn"
    );
}

#[test]
fn boundary_instants_use_half_open_windows() {
    let entity = uuid::Uuid::from_u128(6);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01" :valid-to "2022-01-01"}} [[#uuid "{entity}" :status/value :window]])"#
    ))
    .expect("bounded window");

    let at_start = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2020_07,
        T_2021_01,
        10,
    );
    assert_eq!(at_start.rows.len(), 1, "valid_from == t2 is visible at t2");
    assert_eq!(at_start.rows[0].change, ValidTimeDiffChange::Appeared);

    let at_end = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2021_01,
        T_2022_01,
        10,
    );
    assert_eq!(at_end.rows.len(), 1, "valid_to == t2 is not visible at t2");
    assert_eq!(at_end.rows[0].change, ValidTimeDiffChange::Disappeared);
}

#[test]
fn backdated_corrections_and_view_pinning() {
    let entity = uuid::Uuid::from_u128(7);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2019-01-01"}} [[#uuid "{entity}" :status/value :backdated]])"#
    ))
    .expect("backdated assertion");

    let view = view_any(&db);
    let old_instants = run_diff(
        &view,
        ":status/value",
        Some(&[entity]),
        T_2018_07,
        T_2019_07,
        10,
    );
    assert_eq!(
        old_instants.rows.len(),
        1,
        "a recent commit with an old valid window must appear in an old-instant diff"
    );
    assert_eq!(old_instants.rows[0].change, ValidTimeDiffChange::Appeared);
    assert_eq!(old_instants.rows[0].valid_from, T_2019_01);

    db.execute(&format!(
        r#"(retract [[#uuid "{entity}" :status/value :backdated]])"#
    ))
    .expect("post-view retraction");
    db.execute(&format!(
        r#"(transact {{:valid-from "2019-01-01"}} [[#uuid "{entity}" :status/value :late]])"#
    ))
    .expect("post-view assertion");

    let pinned = run_diff(
        &view,
        ":status/value",
        Some(&[entity]),
        T_2018_07,
        T_2019_07,
        10,
    );
    assert_eq!(pinned.rows.len(), 1, "pinned view ignores later writes");
    assert_eq!(
        pinned.rows[0].value,
        Value::Keyword(":backdated".to_owned())
    );

    let fresh = run_diff(
        &view_any(&db),
        ":status/value",
        Some(&[entity]),
        T_2018_07,
        T_2019_07,
        10,
    );
    assert_eq!(fresh.rows.len(), 1, "fresh view sees the correction");
    assert_eq!(fresh.rows[0].value, Value::Keyword(":late".to_owned()));
}

#[test]
fn committed_delta_and_pending_layers_merge_in_diff() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("valid-time-diff.graph");
    let entity = uuid::Uuid::from_u128(8);
    {
        let db = Minigraf::open(&path).expect("open database");
        db.execute(&format!(
            r#"(transact {{:valid-from "2020-01-01"}} [[#uuid "{entity}" :card/tag :base]])"#
        ))
        .expect("base write");
        db.checkpoint().expect("base checkpoint");
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :delta]])"#
        ))
        .expect("delta write");
        db.checkpoint().expect("delta checkpoint");
    }
    let db = Minigraf::open(&path).expect("reopen database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2022-01-01"}} [[#uuid "{entity}" :card/tag :pending]])"#
    ))
    .expect("pending write");

    let page = run_diff(
        &view_any(&db),
        ":card/tag",
        Some(&[entity]),
        T_2019_07,
        T_2022_07,
        10,
    );
    assert_eq!(page.rows.len(), 3, "base, delta, and pending values appear");
    assert!(
        page.rows
            .iter()
            .all(|row| row.change == ValidTimeDiffChange::Appeared)
    );
    for keyword in [":base", ":delta", ":pending"] {
        assert!(
            page.rows
                .iter()
                .any(|row| row.value == Value::Keyword(keyword.to_owned())),
            "layer value present"
        );
    }
}

#[test]
fn entity_set_scope_preserves_request_order_and_dedup() {
    let first = uuid::Uuid::from_u128(20);
    let second = uuid::Uuid::from_u128(10);
    let silent = uuid::Uuid::from_u128(30);
    let db = Minigraf::in_memory().expect("in-memory database");
    for entity in [first, second] {
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :hit]])"#
        ))
        .expect("scored write");
    }

    let page = run_diff(
        &view_any(&db),
        ":card/tag",
        Some(&[first, silent, second, first]),
        T_2020_07,
        T_2021_07,
        10,
    );
    assert_eq!(page.rows.len(), 2, "silent entity contributes no rows");
    assert_eq!(page.rows[0].entity, first, "request order is preserved");
    assert_eq!(page.rows[1].entity, second);
}

#[test]
fn attribute_scope_visits_entities_in_index_order() {
    let low = uuid::Uuid::from_u128(1);
    let high = uuid::Uuid::from_u128(2);
    let db = Minigraf::in_memory().expect("in-memory database");
    for entity in [high, low] {
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :hit]])"#
        ))
        .expect("scored write");
    }

    let page = run_diff(&view_any(&db), ":card/tag", None, T_2020_07, T_2021_07, 10);
    assert_eq!(page.rows.len(), 2);
    assert_eq!(
        page.rows[0].entity, low,
        "attribute scope follows index order"
    );
    assert_eq!(page.rows[1].entity, high);
}

#[test]
fn attribute_scope_continuation_pages_without_loss_or_duplication() {
    let db = Minigraf::in_memory().expect("in-memory database");
    let entities = (1u128..=12).map(uuid::Uuid::from_u128).collect::<Vec<_>>();
    for entity in &entities {
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :hit]])"#
        ))
        .expect("scored write");
    }
    let view = view_any(&db);

    let mut seen = Vec::new();
    let mut after = None;
    let mut pages = 0usize;
    loop {
        let page = view
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: None,
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: after.as_ref(),
                limit: 5,
            })
            .expect("diff page");
        pages += 1;
        assert!(page.rows.len() <= 5);
        seen.extend(page.rows.iter().map(|row| row.entity));
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
        assert!(pages < 10, "pagination must terminate");
    }
    assert_eq!(pages, 3, "12 single-row entities page as 5 + 5 + 2");
    assert_eq!(seen.len(), 12, "no row is lost");
    let mut deduped = seen.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(deduped.len(), 12, "no row is duplicated");
}

#[test]
fn entity_set_continuation_pages_without_loss_or_duplication() {
    let db = Minigraf::in_memory().expect("in-memory database");
    let entities = (1u128..=12).map(uuid::Uuid::from_u128).collect::<Vec<_>>();
    for entity in &entities {
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :hit]])"#
        ))
        .expect("scored write");
    }
    let view = view_any(&db);

    let mut seen = Vec::new();
    let mut after = None;
    loop {
        let page = view
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: Some(&entities),
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: after.as_ref(),
                limit: 5,
            })
            .expect("diff page");
        seen.extend(page.rows.iter().map(|row| row.entity));
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
        assert!(seen.len() <= 24, "pagination must terminate");
    }
    assert_eq!(seen, entities, "entity-set pages preserve request order");
}

#[test]
fn continuation_is_rejected_for_a_different_request_or_view() {
    let db = Minigraf::in_memory().expect("in-memory database");
    let entities = (1u128..=12).map(uuid::Uuid::from_u128).collect::<Vec<_>>();
    for entity in &entities {
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :hit]])"#
        ))
        .expect("scored write");
    }
    let view = view_any(&db);
    let first = view
        .valid_time_diff(ValidTimeDiffRequest {
            attribute: ":card/tag",
            entities: None,
            valid_at_before: T_2020_07,
            valid_at_after: T_2021_07,
            after: None,
            limit: 5,
        })
        .expect("first page");
    let cursor = first.next.expect("continuation");

    assert!(
        view.valid_time_diff(ValidTimeDiffRequest {
            attribute: ":card/tag",
            entities: None,
            valid_at_before: T_2020_07,
            valid_at_after: T_2022_07,
            after: Some(&cursor),
            limit: 5,
        })
        .is_err(),
        "different instant pair rejects the cursor"
    );
    assert!(
        view.valid_time_diff(ValidTimeDiffRequest {
            attribute: ":status/value",
            entities: None,
            valid_at_before: T_2020_07,
            valid_at_after: T_2021_07,
            after: Some(&cursor),
            limit: 5,
        })
        .is_err(),
        "different attribute rejects the cursor"
    );
    assert!(
        view.valid_time_diff(ValidTimeDiffRequest {
            attribute: ":card/tag",
            entities: Some(&entities),
            valid_at_before: T_2020_07,
            valid_at_after: T_2021_07,
            after: Some(&cursor),
            limit: 5,
        })
        .is_err(),
        "different scope kind rejects the cursor"
    );

    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{}" :card/tag :late]])"#,
        uuid::Uuid::from_u128(99)
    ))
    .expect("later write");
    let later_view = view_any(&db);
    assert!(
        later_view
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: None,
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: Some(&cursor),
                limit: 5,
            })
            .is_err(),
        "different view transaction rejects the cursor"
    );
}

#[test]
fn single_entity_group_exceeding_limit_fails_closed() {
    let entity = uuid::Uuid::from_u128(40);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :card/tag :one] [#uuid "{entity}" :card/tag :two] [#uuid "{entity}" :card/tag :three] [#uuid "{entity}" :card/tag :four]])"#
    ))
    .expect("four tags");

    assert!(
        view_any(&db)
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: Some(&[entity]),
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: None,
                limit: 3,
            })
            .is_err(),
        "a group larger than the limit is rejected without truncation"
    );
}

#[test]
fn request_validation_rejects_bad_shapes() {
    let entity = uuid::Uuid::from_u128(50);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact [[#uuid "{entity}" :card/tag :hit]])"#
    ))
    .expect("write");
    let view = view_any(&db);

    for (before, after) in [(T_2021_01, T_2021_01), (T_2021_07, T_2021_01)] {
        assert!(
            view.valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: None,
                valid_at_before: before,
                valid_at_after: after,
                after: None,
                limit: 10,
            })
            .is_err(),
            "instants must be strictly increasing"
        );
    }
    for limit in [0usize, 10_001] {
        assert!(
            view.valid_time_diff(ValidTimeDiffRequest {
                attribute: ":card/tag",
                entities: None,
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: None,
                limit,
            })
            .is_err(),
            "limit out of range is rejected"
        );
    }
    for attribute in ["card-tag", ":db/tx-count"] {
        assert!(
            view.valid_time_diff(ValidTimeDiffRequest {
                attribute,
                entities: None,
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: None,
                limit: 10,
            })
            .is_err(),
            "attribute must be a stored namespaced keyword"
        );
    }
    let empty: [uuid::Uuid; 0] = [];
    assert!(
        view.valid_time_diff(ValidTimeDiffRequest {
            attribute: ":card/tag",
            entities: Some(&empty),
            valid_at_before: T_2020_07,
            valid_at_after: T_2021_07,
            after: None,
            limit: 10,
        })
        .is_err(),
        "empty entity set is rejected"
    );
    let too_many = (0u128..129).map(uuid::Uuid::from_u128).collect::<Vec<_>>();
    assert!(
        view.valid_time_diff(ValidTimeDiffRequest {
            attribute: ":card/tag",
            entities: Some(&too_many),
            valid_at_before: T_2020_07,
            valid_at_after: T_2021_07,
            after: None,
            limit: 10,
        })
        .is_err(),
        "more than 128 entities is rejected"
    );
    for valid_at in [ReadViewValidAt::Now, ReadViewValidAt::Timestamp(T_2021_01)] {
        let pinned = db
            .read_view(ReadViewOptions {
                as_of: None,
                valid_at,
            })
            .expect("pinned view");
        assert!(
            pinned
                .valid_time_diff(ValidTimeDiffRequest {
                    attribute: ":card/tag",
                    entities: None,
                    valid_at_before: T_2020_07,
                    valid_at_after: T_2021_07,
                    after: None,
                    limit: 10,
                })
                .is_err(),
            "valid-time-pinned views are rejected"
        );
    }
}

#[test]
fn all_value_types_round_trip() {
    let entity = uuid::Uuid::from_u128(60);
    let target = uuid::Uuid::from_u128(61);
    let db = Minigraf::in_memory().expect("in-memory database");
    db.execute(&format!(
        r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :mixed/value "text"] [#uuid "{entity}" :mixed/value 42] [#uuid "{entity}" :mixed/value 1.5] [#uuid "{entity}" :mixed/value true] [#uuid "{entity}" :mixed/value #uuid "{target}"] [#uuid "{entity}" :mixed/value :kw/tag]])"#
    ))
    .expect("mixed transaction");

    let page = run_diff(
        &view_any(&db),
        ":mixed/value",
        Some(&[entity]),
        T_2020_07,
        T_2021_07,
        10,
    );
    assert_eq!(page.rows.len(), 6);
    assert!(
        page.rows
            .iter()
            .all(|row| row.change == ValidTimeDiffChange::Appeared)
    );
    for expected in [
        Value::String("text".to_owned()),
        Value::Integer(42),
        Value::Float(1.5),
        Value::Boolean(true),
        Value::Ref(target),
        Value::Keyword(":kw/tag".to_owned()),
    ] {
        assert!(
            page.rows.iter().any(|row| row.value == expected),
            "value variant present"
        );
    }
}

#[test]
fn source_entry_budget_fails_closed() {
    let entity = uuid::Uuid::from_u128(70);
    let db = Minigraf::in_memory().expect("in-memory database");
    for chunk in 0..7 {
        let facts = (0..10_000)
            .map(|index| {
                format!(
                    r#"[#uuid "{entity}" :bulk/value {}]"#,
                    chunk * 10_000 + index
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [{facts}])"#
        ))
        .expect("bulk transaction");
    }

    assert!(
        view_any(&db)
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":bulk/value",
                entities: Some(&[entity]),
                valid_at_before: T_2019_07,
                valid_at_after: T_2020_07,
                after: None,
                limit: 10,
            })
            .is_err(),
        "history above the source-entry budget is rejected even when no row changes"
    );
}

#[test]
fn page_bytes_budget_fails_closed() {
    let entity = uuid::Uuid::from_u128(80);
    let payload = "v".repeat(1_700);
    let db = Minigraf::in_memory().expect("in-memory database");
    for chunk in 0..5 {
        let facts = (0..1_000)
            .map(|index| {
                format!(r#"[#uuid "{entity}" :chunk/body "{chunk}-{index:04}-{payload}"]"#)
            })
            .collect::<Vec<_>>()
            .join(" ");
        db.execute(&format!(
            r#"(transact {{:valid-from "2021-01-01"}} [{facts}])"#
        ))
        .expect("large transaction");
    }

    assert!(
        view_any(&db)
            .valid_time_diff(ValidTimeDiffRequest {
                attribute: ":chunk/body",
                entities: Some(&[entity]),
                valid_at_before: T_2020_07,
                valid_at_after: T_2021_07,
                after: None,
                limit: 10_000,
            })
            .is_err(),
        "a page above the byte budget is rejected without truncation"
    );
}

/// 1M measurement receipt for the A-2 gate. Run with:
/// `cargo test --test valid_time_diff_test --release -- --ignored --nocapture`
#[test]
#[ignore = "1M measurement receipt; run explicitly in release mode"]
fn measure_valid_time_diff_1m() {
    const BASE_FACTS: usize = 1_000_000;
    const BATCH: usize = 1_000;
    const RECEIPT_ENTITIES: usize = 128;
    const SAMPLES: usize = 20;

    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("valid-time-diff-1m.graph");
    {
        let db = minigraf::OpenOptions {
            wal_checkpoint_threshold: usize::MAX,
            ..Default::default()
        }
        .path(&path)
        .open()
        .expect("open database");
        for batch_start in (0..BASE_FACTS).step_by(BATCH) {
            let mut command = String::from("(transact [");
            for index in batch_start..batch_start + BATCH {
                let entity = uuid::Uuid::from_u128(0x1000_0000 + index as u128);
                command.push_str(&format!(r#"[#uuid "{entity}" :bulk/noise {index}] "#));
            }
            command.push_str("])");
            db.execute(&command).expect("base batch");
        }
        let receipt_entities = (0..RECEIPT_ENTITIES)
            .map(|index| uuid::Uuid::from_u128(0x9000_0000 + index as u128))
            .collect::<Vec<_>>();
        for entity in &receipt_entities {
            db.execute(&format!(
                r#"(transact {{:valid-from "2020-01-01" :valid-to "2021-01-01"}} [[#uuid "{entity}" :status/value :old]])"#
            ))
            .expect("old window");
            db.execute(&format!(
                r#"(transact {{:valid-from "2021-01-01"}} [[#uuid "{entity}" :status/value :new]])"#
            ))
            .expect("new window");
        }
        db.checkpoint().expect("checkpoint");
    }

    let db = minigraf::OpenOptions {
        wal_checkpoint_threshold: usize::MAX,
        ..Default::default()
    }
    .path(&path)
    .open()
    .expect("reopen database");
    let receipt_entities = (0..RECEIPT_ENTITIES)
        .map(|index| uuid::Uuid::from_u128(0x9000_0000 + index as u128))
        .collect::<Vec<_>>();
    let view = view_any(&db);

    let measure = |label: &str, entities: Option<&[uuid::Uuid]>| {
        let mut samples = Vec::with_capacity(SAMPLES);
        for iteration in 0..=SAMPLES {
            let started = std::time::Instant::now();
            let page = view
                .valid_time_diff(ValidTimeDiffRequest {
                    attribute: ":status/value",
                    entities,
                    valid_at_before: T_2020_07,
                    valid_at_after: T_2021_07,
                    after: None,
                    limit: 1_000,
                })
                .expect("diff page");
            let elapsed = started.elapsed();
            assert_eq!(page.rows.len(), RECEIPT_ENTITIES * 2, "exact diff rows");
            assert!(page.next.is_none());
            if iteration > 0 {
                samples.push(elapsed.as_secs_f64() * 1_000.0);
            }
        }
        samples.sort_by(|left, right| left.total_cmp(right));
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)];
        let max = samples[samples.len() - 1];
        println!("valid_time_diff_1m {label}: p50 {p50:.3} ms, p95 {p95:.3} ms, max {max:.3} ms");
    };

    measure("entity_set_128", Some(&receipt_entities));
    measure("attribute_scope", None);
}

/// Native re-measurement against an externally generated fixture, so a browser
/// receipt and a native number can be attributed to the same file rather than
/// to two differently shaped fixtures. Generate with:
/// `cargo run --release --example generate_bench_fixture -- 1000000 <out.graph> 128`
/// then run:
/// `VICIA_DIFF_FIXTURE=<out.graph> cargo test --test valid_time_diff_test --release -- --ignored measure_valid_time_diff_fixture --nocapture`
#[test]
#[ignore = "external fixture measurement; run explicitly in release mode"]
fn measure_valid_time_diff_fixture() {
    const RECEIPT_ENTITY_BASE: u128 = 0x9000_0000;
    const SAMPLES: usize = 20;

    let fixture = std::env::var("VICIA_DIFF_FIXTURE")
        .expect("set VICIA_DIFF_FIXTURE to a generate_bench_fixture output");
    let receipt_entity_count: usize = std::env::var("VICIA_DIFF_ENTITIES")
        .map(|value| value.parse().expect("VICIA_DIFF_ENTITIES must be a number"))
        .unwrap_or(128);

    // Copy before opening: open may write a WAL sidecar or migrate in place,
    // and the fixture hash recorded in a receipt must stay stable.
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("fixture.graph");
    std::fs::copy(&fixture, &path).expect("copy fixture");

    let db = minigraf::OpenOptions {
        wal_checkpoint_threshold: usize::MAX,
        ..Default::default()
    }
    .path(&path)
    .open()
    .expect("open fixture");
    let receipt_entities = (0..receipt_entity_count)
        .map(|index| uuid::Uuid::from_u128(RECEIPT_ENTITY_BASE + index as u128))
        .collect::<Vec<_>>();
    let view = view_any(&db);

    let measure = |label: &str, entities: Option<&[uuid::Uuid]>| {
        let mut samples = Vec::with_capacity(SAMPLES);
        for iteration in 0..=SAMPLES {
            let started = std::time::Instant::now();
            let page = run_diff(
                &view,
                ":status/value",
                entities,
                T_2020_07,
                T_2021_07,
                1_000,
            );
            let elapsed = started.elapsed();
            assert_eq!(page.rows.len(), receipt_entity_count * 2, "exact diff rows");
            assert!(page.next.is_none(), "no continuation expected");
            if iteration > 0 {
                samples.push(elapsed.as_secs_f64() * 1_000.0);
            }
        }
        samples.sort_by(|left, right| left.total_cmp(right));
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)];
        let max = samples[samples.len() - 1];
        println!(
            "valid_time_diff_fixture {label}: p50 {p50:.3} ms, p95 {p95:.3} ms, max {max:.3} ms"
        );
    };

    measure("entity_set", Some(&receipt_entities));
    measure("attribute_scope", None);
}
