use super::*;
use cargo_gpui_visual::{
    report::{Group, Options},
    terminal::{self, Paths},
};

fn summary(f: &Fixture, review: &Review) -> String {
    terminal::summary(
        review,
        Paths {
            repo: &f.repo,
            run: &f.run,
            baselines: &f.base,
            report: &f.report,
        },
        false,
    )
}

#[test]
fn unchanged_stills_keep_comparison_sliders_in_gallery() {
    let f = Fixture::new();
    f.establish();
    let r = f.compare().unwrap();
    assert!(r.passed);
    let doc = f.doc();
    assert!(doc.contains("class=\"group global-mode\" hidden"));
    let gallery = doc.split("<h1 id=\"alle\">").nth(1).unwrap();
    for case in &f.cases {
        assert!(gallery.contains(&format!("name=\"mode-{case}\"")));
        assert!(gallery.contains(&format!("src=\"{case}/expected.png\"")));
        assert_eq!(doc.matches(&format!("id=\"{case}-swipe\"")).count(), 1);
    }
    assert_eq!(
        gallery.matches("class=\"handle\" role=\"slider\"").count(),
        2
    );
    assert_eq!(gallery.matches("class=\"blend\" type=\"range\"").count(), 2);
    assert!(gallery.contains("loading=\"lazy\""));
}

#[test]
fn unchanged_stills_and_new_sequences_keep_global_controls_without_fake_references() {
    let mut f = sequences::sequences();
    let journeys = load_manifest(&f.run).unwrap().sequences;
    for theme in ["light", "dark"] {
        let id = format!("screen-proof--{theme}");
        f.cases.push(id.clone());
        f.image(&id, 2, 3, [30, 40, 50, 255]);
    }
    f.seal();
    f.manifest(|m| m["sequences"] = json!(journeys));
    f.compare().unwrap();
    f.accept(
        &["screen-proof--light".into(), "screen-proof--dark".into()],
        false,
    )
    .unwrap();
    let r = f.compare().unwrap();
    assert_eq!(
        r.results.iter().filter(|r| r.status == "changed").count(),
        0
    );
    let doc = f.doc();
    assert!(doc.contains("class=\"group global-mode\" hidden"));
    assert!(doc.contains("name=\"mode-screen-proof--light\""));
    assert!(doc.contains("id=\"sequence-story-60hz--light-swipe\" disabled"));
    assert!(doc.contains("class=\"comparison-unavailable hint\""));
    assert!(doc.contains("class=seq-slider"));
    assert!(!doc.contains("sequence-expected/"));
    assert!(!doc.contains("sequence-story-60hz-frame-0000--light/expected.png"));
    retain_fixture(&f, &r, "GPUI_UNCHANGED_REVIEW_OUTPUT");
}

#[test]
fn gallery_groups_are_consumer_defined_and_escaped() {
    let f = Fixture::new();
    let r = f.compare().unwrap();
    let options = Options {
        groups: vec![Group {
            title: "Widgets <&>".into(),
            prefixes: vec!["component-".into()],
        }],
    };
    report::write_with_options(&f.report, &r, &options).unwrap();
    let doc = f.doc();
    assert!(doc.contains("Widgets &lt;&amp;&gt; <span class=\"count\">1</span>"));
    assert!(doc.contains("neu · keine Referenz"));
    assert!(doc.contains("Abweichungen <span class=count>2</span>"));
    assert!(!doc.contains("Feed & Frontpage"));
    let bad = Options {
        groups: vec![Group {
            title: "bad".into(),
            prefixes: vec![String::new()],
        }],
    };
    assert!(report::write_with_options(&f.report, &r, &bad).is_err());
}

#[test]
fn compact_summary_lists_whole_journeys_and_never_frames() {
    let f = sequences::sequences();
    let r = f.compare().unwrap();
    let text = summary(&f, &r);
    assert!(text.contains("2 ohne Referenz von 2"));
    assert!(text.contains("4 Bilder · 2 Sequenzen"));
    assert!(!text.contains("-frame-"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("file://"));
    assert!(text.contains("--run '"));
    assert!(text.contains("--report '"));
    assert!(text.contains("--case 'sequence-story-60hz--dark'"));
    assert_eq!(text.matches("--case").count(), 1);
    assert!(!text.contains("make gpui-visual"));
}

#[test]
fn timeline_only_drift_is_a_visible_summary_finding() {
    let f = sequences::sequences();
    sequences::establish(&f);
    f.manifest(|m| m["sequences"][0]["ticks"][2]["events"] = json!(["late"]));
    let r = f.compare().unwrap();
    assert!(r.results.iter().all(|r| r.status == "same"));
    let text = summary(&f, &r);
    assert!(text.contains("1 geändert, 0 ohne Referenz"));
    assert!(text.contains("erste Abweichung: Takt 2"));
    assert!(text.contains("--case 'sequence-story-60hz--light'"));
}

#[test]
fn terminal_caps_findings_and_blocks_invalid_acceptance_hints() {
    let mut f = Fixture::new();
    for n in 0..20 {
        for theme in ["light", "dark"] {
            let id = format!("component-extra-{n}--{theme}");
            f.image(&id, 2, 3, [0; 4]);
            f.cases.push(id);
        }
    }
    f.seal();
    let mut r = f.compare().unwrap();
    let text = summary(&f, &r);
    assert!(text.contains("… 30 weitere im Bericht"));
    assert!(text.lines().count() < 22);
    r.environment_mismatch = Some(json!({"baselines":{"removed":1},"capture":{"new":1}}));
    let text = summary(&f, &r);
    assert!(text.contains("new, removed"));
    assert!(!text.contains("Übernehmen"));
    r.errors = vec!["broken metadata".into()];
    assert!(summary(&f, &r).contains("ungültige Eingabe"));
    assert!(!summary(&f, &r).contains("--case"));
}

#[test]
fn sequence_viewer_preserves_modes_placeholders_filter_and_tail() {
    let f = sequences::sequences();
    f.compare().unwrap();
    let doc = f.doc();
    assert!(doc.contains("keine Referenz"));
    assert!(doc.contains("data-comparable=\"false\""));
    assert!(!doc.contains("<img loading=lazy src="));
    assert!(doc.contains("<button class=\"tick new"));
    assert!(!doc.contains("<button class=\"tick same"));
    sequences::establish(&f);
    let mut r = f.compare().unwrap();
    assert!(f.doc().contains("class=\"sequence quiet\""));
    r.sequences[0].actual.ticks.pop();
    r.sequences[0].first_difference = Some(2);
    r.passed = false;
    report::write(&f.report, &r).unwrap();
    let doc = f.doc();
    assert!(doc.contains("class=\"group global-mode\" hidden"));
    assert!(doc.contains("name=\"mode-sequence-story-60hz--light\""));
    assert!(doc.contains("data-image=\"expected\""));
    assert!(doc.contains("data-link=\"diff\""));
    assert!(doc.contains("im Kandidaten fehlend"));
    assert!(doc.contains("class=\"sequence finding\""));
    assert!(doc.contains("data-tick=\"2\" data-actual=\"\""));
    // The always-visible tick strip names the missing tail, not a blank diff.
    assert!(doc.contains("<button class=\"tick missing\""));
    assert!(doc.contains("<button class=\"tick same"));
    assert!(!doc.contains("<button class=\"tick new"));
    assert_eq!(doc.matches("<script").count(), 1);
    assert_eq!(r.sequences.len(), 2);
}

#[test]
fn dimension_changes_highlight_measurements_in_header_and_inspector_only() {
    let f = Fixture::new();
    f.compare().unwrap();
    assert!(!f.doc().contains("class=\"dimension-change\""));
    f.establish();
    assert!(!f.doc().contains("class=\"dimension-change\""));
    f.image(&f.cases[0], 24, 16, [20, 30, 40, 255]);
    f.image(&f.cases[1], 20, 12, [30, 40, 50, 255]);
    f.seal();
    let mut review = f.compare().unwrap();
    let highlighted = "<span class=\"dimension-change\">Bildgröße 20×12 → 24×16</span>";
    let doc = f.doc();
    assert!(doc.contains(&format!("<p class=\"case-meta\">{highlighted}</p>")));
    assert!(doc.contains(&format!("<p class=\"panel-note\">{highlighted}.")));
    assert_eq!(doc.matches("class=\"dimension-change\"").count(), 2);
    assert!(doc.contains(".dimension-change{color:var(--color-changed-text)}"));
    retain_fixture(&f, &review, "GPUI_DIMENSION_REVIEW_OUTPUT");

    // Missing dimension metadata keeps the semantic highlight without inventing a size.
    let item = review
        .results
        .iter_mut()
        .find(|i| i.reason.as_deref() == Some("dimensions"))
        .unwrap();
    item.expected_size = None;
    report::write(&f.report, &review).unwrap();
    assert_eq!(
        f.doc()
            .matches("<span class=\"dimension-change\">Bildgröße geändert</span>")
            .count(),
        2
    );
}

#[test]
fn inspector_offers_exact_hints_only_for_valid_full_runs() {
    let mut f = Fixture::new();
    f.establish();
    // Two separate changed row bands in one case.
    let case = f.cases[0].clone();
    let path = f.run.join(format!("{case}.png"));
    let mut image = read_png(&path).unwrap();
    for x in 2..6 {
        image.put_pixel(x, 1, Rgba([255, 0, 0, 255]));
        image.put_pixel(x, 11, Rgba([0, 0, 255, 255]));
    }
    image.save(&path).unwrap();
    f.seal();
    let r = f.compare().unwrap();
    let finding = r.results.iter().find(|i| i.case == case).unwrap();
    assert_eq!(finding.rows, vec![[1, 2], [11, 12]]);
    let doc = f.doc();
    let section = doc
        .split(&format!("id=\"f-{case}\""))
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    assert_eq!(section.matches("<details class=\"region\"").count(), 2);
    assert!(section.contains("<details class=\"region\" open>"));
    assert!(section.contains("y 11–12"));
    assert!(section.contains(&format!("--case &#39;{case}&#39;")));
    assert!(section.contains(&format!("--run &#39;{}&#39;", f.run.display())));
    assert!(section.contains(&format!("--baselines &#39;{}&#39;", f.base.display())));
    assert_eq!(doc.matches("class=\"command\"").count(), 1);
    // Every entry stays reachable from the table of contents.
    assert!(doc.contains(&format!("href=\"#f-{case}\"")));
    for other in &f.cases[1..] {
        assert!(doc.contains(&format!("href=\"#s-{other}\"")));
        assert!(doc.contains(&format!("id=\"s-{other}\"")));
    }
    // A direct write lacks the comparison paths and prints no command.
    report::write(&f.report, &r).unwrap();
    assert!(!f.doc().contains("class=\"command\""));
    assert!(f
        .doc()
        .contains("steht in der Ausgabe von check oder compare"));
    // Environment drift blocks the hint, exactly like the terminal summary.
    f.env["os"] = json!("drifted-os");
    f.seal();
    f.compare().unwrap();
    let doc = f.doc();
    assert!(doc.contains("Aufnahmeumgebung weicht ab"));
    assert!(doc.contains("Umgebung weicht ab</a>"));
    assert!(!doc.contains("class=\"command\""));
    assert!(!doc.contains("cargo gpui-visual accept \\"));
}

/// Optional retained synthetic report for managed-browser QA, never product evidence.
#[test]
fn review_parity_fixture() {
    let mut f = sequences::sequences();
    for id in ["component-proof--light", "component-proof--dark"] {
        f.cases.push(id.into());
        f.image(id, 804, 1748, [235, 238, 242, 255]);
    }
    for id in &f.cases {
        if id.starts_with("sequence-") {
            let dark = id.ends_with("--dark");
            f.image(
                id,
                402,
                874,
                if dark {
                    [22, 26, 33, 255]
                } else {
                    [235, 238, 242, 255]
                },
            );
        }
    }
    f.env["purpose"] =
        json!("SYNTHETIC REPORT CONTROLS — not product capture or approval evidence");
    let journeys = load_manifest(&f.run).unwrap().sequences;
    f.seal();
    f.manifest(|m| m["sequences"] = json!(journeys));
    f.compare().unwrap();
    let mut selection = sequences::ids(&f);
    selection.extend([
        "component-proof--light".into(),
        "component-proof--dark".into(),
    ]);
    f.accept(&selection, false).unwrap();
    for id in f.cases.clone() {
        let p = f.run.join(format!("{id}.png"));
        let mut image = read_png(&p).unwrap();
        for y in 0..image.height() {
            for x in 0..image.width() {
                if (30..50).contains(&y)
                    || (900..915).contains(&y)
                    || (id.starts_with("sequence-") && x == 0)
                {
                    image.put_pixel(x, y, Rgba([60, 100, 180, 255]));
                }
            }
        }
        image.save(&p).unwrap();
        f.manifest(|m| m["images"][&id] = json!(digest(&p).unwrap()));
    }
    f.manifest(|m| {
        m["sequences"][1]["ticks"].as_array_mut().unwrap().pop();
    });
    for theme in ["light", "dark"] {
        for category in ["new", "quiet"] {
            let id = format!("component-{category}--{theme}");
            f.cases.push(id.clone());
            f.image(&id, 402, 874, [38, 50, 66, 255]);
        }
    }
    let journeys = load_manifest(&f.run).unwrap().sequences;
    f.seal();
    f.manifest(|m| m["sequences"] = json!(journeys));
    f.compare().unwrap();
    f.accept(
        &[
            "component-quiet--light".into(),
            "component-quiet--dark".into(),
        ],
        false,
    )
    .unwrap();
    let r = f.compare().unwrap();
    let options = Options {
        groups: vec![Group {
            title: "Prüfmuster".into(),
            prefixes: vec!["component-".into()],
        }],
    };
    report::write_with_options(&f.report, &r, &options).unwrap();
    retain_fixture(&f, &r, "GPUI_REVIEW_FIXTURE_OUTPUT");
}

fn retain_fixture(f: &Fixture, r: &Review, variable: &str) {
    if let Some(target) = std::env::var_os(variable) {
        let target = PathBuf::from(target);
        assert!(!target.exists(), "fresh report directory required");
        fn copy(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for entry in fs::read_dir(from).unwrap() {
                let p = entry.unwrap().path();
                let out = to.join(p.file_name().unwrap());
                if p.is_dir() {
                    copy(&p, &out)
                } else {
                    fs::copy(p, out).unwrap();
                }
            }
        }
        copy(&f.report, &target);
        fs::write(target.join("terminal.txt"), summary(f, r)).unwrap();
    }
}

#[test]
fn terminal_quotes_paths_encodes_links_and_uses_color_only_when_requested() {
    let f = Fixture::new();
    let r = f.compare().unwrap();
    let path = Path::new("/tmp/space and 'quote#ü");
    let text = terminal::summary(
        &r,
        Paths {
            repo: path,
            run: path,
            baselines: path,
            report: path,
        },
        false,
    );
    assert!(text.contains("file:///tmp/space%20and%20%27quote%23%C3%BC/index.html"));
    assert!(text.contains("--repo '/tmp/space and '\\''quote#ü'"));
    assert!(!text.contains('\u{1b}'));
    assert!(terminal::summary(
        &r,
        Paths {
            repo: path,
            run: path,
            baselines: path,
            report: path
        },
        true
    )
    .contains("\u{1b}[1;31m"));
    f.establish();
    assert!(summary(&f, &f.compare().unwrap()).contains("✓ Vergleich   alle 2 Prüffälle"));
}

#[test]
fn review_configuration_is_optional_validated_and_wired_through_cli() {
    let f = Fixture::new();
    let config = r#"schema = 1
capture = ["unused-producer"]
baselines = "baselines"
report = "report"
runs = "runs"
[source_policy]
scopes = ["renderer.rs"]
suffixes = [".rs"]
"#;
    let call = |command: &str| {
        Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
            .args([command, "--repo"])
            .arg(&f.repo)
            .args(["--run", "run"])
            .output()
            .unwrap()
    };
    fs::write(f.repo.join("visual.toml"), config).unwrap();
    assert!(call("doctor").status.success());
    fs::write(
        f.repo.join("visual.toml"),
        format!(
            "{config}\n[[review.groups]]\ntitle = 'Custom components'\nprefixes = ['component-']\n"
        ),
    )
    .unwrap();
    let output = call("compare");
    assert_eq!(output.status.code(), Some(1));
    assert!(f.doc().contains("Custom components"));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Bericht     file://"));
    assert!(!stdout.contains('\u{1b}'));
    for invalid in [
        "title = ''\nprefixes = ['component-']",
        "title = 'Empty'\nprefixes = ['']",
        "title = 'Empty'\nprefixes = []",
        "title = 'Unknown'\nprefixes = ['component-']\nunknown = true",
    ] {
        fs::write(
            f.repo.join("visual.toml"),
            format!("{config}\n[[review.groups]]\n{invalid}\n"),
        )
        .unwrap();
        assert_eq!(call("doctor").status.code(), Some(2));
    }
}
