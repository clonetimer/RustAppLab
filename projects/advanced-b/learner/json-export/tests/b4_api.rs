use serde_json::{json, Value};
use taskcore::Task;
use taskpocket_json::to_json;

fn task(id: u64, done: bool, title: &str) -> Task {
    Task {
        id,
        done,
        title: title.into(),
    }
}

#[test]
fn empty_snapshot_is_an_empty_json_array() {
    assert_eq!(to_json(&[]).unwrap(), "[]");
}

#[test]
fn fields_have_exact_names_and_json_types() {
    let text = to_json(&[task(7, true, "中文任务")]).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        value,
        json!([{ "id": 7, "done": true, "title": "中文任务" }])
    );
    let item = &value[0];
    assert!(item["id"].is_u64());
    assert!(item["done"].is_boolean());
    assert!(item["title"].is_string());
    assert_eq!(item.as_object().unwrap().len(), 3);
}

#[test]
fn sorts_by_id_and_preserves_both_boolean_values() {
    let value: Value = serde_json::from_str(
        &to_json(&[
            task(9, false, "晚"),
            task(2, true, "早"),
            task(4, false, "中"),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        value,
        json!([
            {"id":2,"done":true,"title":"早"},
            {"id":4,"done":false,"title":"中"},
            {"id":9,"done":false,"title":"晚"},
        ])
    );
}

#[test]
fn u64_max_is_preserved_without_float_rounding() {
    let text = to_json(&[task(u64::MAX, false, "最大编号")]).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value[0]["id"].as_u64(), Some(u64::MAX));
    assert!(text.contains("18446744073709551615"));
}

#[test]
fn escapes_quotes_backslashes_and_internal_legal_controls() {
    let title = "中文\"引号\\路径 中\0间\u{0008}字";
    let text = to_json(&[task(1, false, title)]).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value[0]["title"].as_str(), Some(title));
    assert!(!text.contains('\0'));
    assert!(!text.contains('\u{0008}'));
}

#[test]
fn exporting_does_not_mutate_the_input_order_or_values() {
    let tasks = vec![task(6, true, "六"), task(1, false, "一")];
    let before = tasks.clone();
    to_json(&tasks).unwrap();
    assert_eq!(tasks, before);
}

#[test]
fn rejects_zero_id() {
    assert!(to_json(&[task(0, false, "零")]).is_err());
}

#[test]
fn rejects_duplicate_ids_even_when_other_fields_differ() {
    assert!(to_json(&[task(2, false, "甲"), task(2, true, "乙")]).is_err());
}

#[test]
fn rejects_empty_or_blank_titles() {
    for title in ["", " ", "\u{3000}"] {
        assert!(to_json(&[task(1, false, title)]).is_err(), "{title:?}");
    }
}

#[test]
fn rejects_tsv_delimiters_inside_titles() {
    for title in ["中\t间", "中\r间", "中\n间"] {
        assert!(to_json(&[task(1, false, title)]).is_err(), "{title:?}");
    }
}

#[test]
fn rejects_snapshots_that_would_silently_trim() {
    for title in [
        " 前",
        "后 ",
        "\u{3000}中文",
        "中文\u{3000}",
        "\t标题",
        "标题\n",
    ] {
        assert!(to_json(&[task(1, false, title)]).is_err(), "{title:?}");
    }
}

#[test]
fn accepts_eighty_unicode_characters_in_every_feature_mode() {
    let title = "学".repeat(80);
    let value: Value = serde_json::from_str(&to_json(&[task(1, false, &title)]).unwrap()).unwrap();
    assert_eq!(value[0]["title"].as_str(), Some(title.as_str()));
}

#[test]
fn feature_controls_the_eighty_one_character_boundary() {
    let result = to_json(&[task(1, false, &"学".repeat(81))]);
    #[cfg(feature = "strict-titles")]
    assert!(result.is_err());
    #[cfg(not(feature = "strict-titles"))]
    assert!(result.is_ok());
}
