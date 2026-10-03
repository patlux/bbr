use super::*;

#[test]
fn compact_catalog_navigation_grows_with_wrapped_labels() {
    let f = Fixture::new();
    let long_label =
        "History · three undated fixture visits with a deliberately long localized label <&>";
    f.catalog(0, |v| {
        v["snapshots"][0]["label"] = json!(long_label);
        v["snapshots"].as_array_mut().unwrap().push(json!({
            "base": "remaining-surfaces",
            "label": "Other application surfaces and states not yet captured",
            "group": "Coverage still open",
            "state": "withheld",
            "reason": "Not part of this synthetic report fixture"
        }));
    });
    f.seal(0);
    f.compare().unwrap();
    let html = f.html();
    for (case, label, status) in [
        ("panel-page-one", long_label, "missing-baseline"),
        (
            "remaining-surfaces",
            "Other application surfaces and states not yet captured",
            "withheld",
        ),
    ] {
        for theme in ["light", "dark"] {
            assert!(html.contains(&format!(
                "<span class=\"toc-label\">{} · {theme} · {status}</span>",
                report::escape(label)
            )));
            assert!(html.contains(&format!("gallery:{case}--{theme}\"")));
        }
    }
    // Assert the embedded stylesheet, not just the source asset: collection
    // labels must wrap without clipping, ellipsis or a fixed-height hit target.
    let declarations = |selector: &str| -> Vec<&str> {
        html.split(&format!("{selector}{{"))
            .nth(1)
            .unwrap()
            .split('}')
            .next()
            .unwrap()
            .split(';')
            .map(str::trim)
            .collect()
    };
    let row = declarations(".toc-row.compact");
    assert!(row.contains(&"min-height:24px"));
    assert!(row.contains(&"padding:4px 8px"));
    assert!(!row
        .iter()
        .any(|d| d.starts_with("height:") || d.starts_with("max-height:")));
    let label = declarations(".toc-label");
    assert!(label.contains(&"min-width:0"));
    assert!(label.contains(&"overflow-wrap:anywhere"));
    assert!(!label
        .iter()
        .any(|d| d.starts_with("text-overflow:") || d.starts_with("overflow:")));
    if std::env::var_os("GPUI_VISUAL_KEEP_NAV_FIXTURE").is_some() {
        println!("NAV_FIXTURE={}", f._temp.keep().display());
    }
}
