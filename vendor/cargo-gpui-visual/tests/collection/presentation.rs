use super::*;

#[test]
fn collection_detail_windows_keep_suite_assets_and_all_changed_pixels() {
    let mut f = Fixture::new();
    for s in &f.suites {
        RgbaImage::from_pixel(608, 200, Rgba([30, 40, 50, 255]))
            .save(s.run.join("panel-page-one--light.png"))
            .unwrap();
    }
    f.seal(0);
    f.seal(1);
    f.compare().unwrap();
    let ids = [
        "gallery:panel-page-one--light".into(),
        "browser:panel-page-one--light".into(),
    ];
    collection::accept(&f.report, &ids, &[]).unwrap();
    let before: Vec<_> = f
        .suites
        .iter()
        .map(|s| baseline_state(&s.baselines).unwrap())
        .collect();
    for (index, s) in f.suites.iter().enumerate() {
        let path = s.run.join("panel-page-one--light.png");
        let mut image = read_png(&path).unwrap();
        image.put_pixel(
            if index == 0 { 0 } else { 607 },
            199,
            Rgba([200, 0, 0, 255]),
        );
        image.save(path).unwrap();
        f.seal(index);
    }
    f.report = f.root.join("changed");
    let review = f.compare().unwrap();
    let html = f.html();
    for (index, s) in f.suites.iter().enumerate() {
        let section = html
            .split(&format!("id=\"f-{}:panel-page-one--light\"", s.id))
            .nth(1)
            .unwrap()
            .split("</section>")
            .next()
            .unwrap();
        let x = if index == 0 { 0 } else { 304 };
        assert!(section.contains(&format!(
            "data-x=\"{x}\" data-w=\"304\" data-y=\"104\" data-h=\"96\""
        )));
        for file in ["expected", "actual", "diff"] {
            assert!(section.contains(&format!(
                "src=\"suites/{}/panel-page-one--light/{file}.png\"",
                s.id
            )));
        }
        assert_eq!(
            review.suites[index]
                .review
                .as_ref()
                .unwrap()
                .results
                .iter()
                .find(|r| r.case == "panel-page-one--light")
                .unwrap()
                .changed_pixels,
            Some(1)
        );
        assert_eq!(before[index], baseline_state(&s.baselines).unwrap());
    }
}

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
