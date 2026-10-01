use cargo_gpui_visual::{
    compare::{Finding, Review},
    inspect::{inspect, Options},
    sequence::{Sequence, SequenceReview, Tick},
    store,
};
use serde_json::json;
use std::fs;

fn review() -> Review {
    let tick = |i| Tick {
        tick: i,
        time_ns: i * 1_000_000_000 / 60,
        drawn: i == 0,
        image: "sequence-opening-frame-0000--light".into(),
        events: vec![],
        state: json!({"ready":true}),
        dirty_before_callbacks: false,
        callbacks_pending: false,
    };
    let actual = Sequence {
        id: "sequence-opening--light".into(),
        hz: 60,
        ticks: vec![tick(0), tick(1)],
    };
    let mut expected = actual.clone();
    expected.ticks[1].state = json!({"ready":false});
    Review {
        schema: 1,
        capture_hash: "stored".into(),
        baseline_state: Default::default(),
        environment: json!({"layer":"gpui"}),
        environment_mismatch: None,
        results: vec![Finding {
            case: "sequence-opening-frame-0000--light".into(),
            status: "same".into(),
            changed_pixels: Some(0),
            bounds: None,
            actual_size: Some([1, 1]),
            expected_size: Some([1, 1]),
            rows: vec![],
            reason: None,
        }],
        errors: vec![],
        passed: false,
        selection: None,
        sequences: vec![SequenceReview {
            actual,
            expected: Some(expected),
            first_difference: Some(1),
        }],
    }
}
#[test]
fn whole_sequence_details_load_one_tick_and_reject_symlink_assets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    store::write_json(&root.join("review.json"), &review()).unwrap();
    let options = Options {
        case: Some("sequence-opening--light".into()),
        ..Default::default()
    };
    let value = inspect(&root, &options).unwrap();
    assert_eq!(value["detail"]["tick"], 1);
    assert_eq!(value["detail"]["actual"]["state"]["ready"], true);
    assert_eq!(value["detail"]["expected"]["state"]["ready"], false);
    assert!(value["detail"]["actual"]["ticks"].is_null());
    assert_eq!(
        inspect(&root, &Options::default()).unwrap()["total_findings"],
        1
    );
    assert!(inspect(
        &root,
        &Options {
            tick: Some(2),
            ..options.clone()
        }
    )
    .is_err());
    #[cfg(unix)]
    {
        let folder = root.join("sequence-opening-frame-0000--light");
        fs::create_dir(&folder).unwrap();
        std::os::unix::fs::symlink(root.join("review.json"), folder.join("actual.png")).unwrap();
        assert!(inspect(&root, &options).is_err());
    }
}
#[test]
fn collection_namespaces_remain_explicit_and_invalid_suites_visible() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let suite = |id, review, errors| json!({"suite":{"id":id,"label":id,"repo":"/unused","run":"run","baselines":"refs","catalog":"catalog.json"},"catalog":null,"catalog_hash":null,"review":review,"errors":errors,"passed":false});
    store::write_json(&root.join("collection.json"), &json!({"schema":1,"passed":false,"suites":[suite("gallery",json!(review()),json!([])),suite("broken",json!(null),json!(["missing coverage"]))]})).unwrap();
    let summary = inspect(&root, &Options::default()).unwrap();
    assert_eq!(summary["suites"][1]["error_count"], 1);
    let mut options = Options {
        case: Some("gallery:sequence-opening--light".into()),
        ..Default::default()
    };
    assert_eq!(inspect(&root, &options).unwrap()["suite"], "gallery");
    options.case = Some("sequence-opening--light".into());
    assert!(inspect(&root, &options).is_err());
    options.case = Some("broken:demo--light".into());
    assert!(inspect(&root, &options).is_err());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
}
