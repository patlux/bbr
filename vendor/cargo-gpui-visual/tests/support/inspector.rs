use super::*;

fn crops(section: &str) -> Vec<[u32; 4]> {
    section
        .split("class=\"to-stage\"")
        .skip(1)
        .map(|tag| {
            let attr = |name| {
                tag.split(&format!("data-{name}=\""))
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap()
                    .parse::<u32>()
                    .unwrap()
            };
            [attr("x"), attr("y"), attr("w"), attr("h")]
        })
        .collect()
}

#[test]
fn inspector_crops_follow_changes_instead_of_the_empty_union_corner() {
    let f = Fixture::new();
    for case in &f.cases {
        f.image(case, 608, 2814, [20, 30, 40, 255]);
    }
    f.seal();
    f.establish();
    let baseline = baseline_state(&f.base).unwrap();
    let case = &f.cases[0];
    let path = f.run.join(format!("{case}.png"));
    let mut image = read_png(&path).unwrap();
    // A full-height edge merges the rows. Unrelated navigation and footer
    // changes make the union's upper left corner entirely unchanged.
    for y in 0..2814 {
        image.put_pixel(600, y, Rgba([200, 30, 40, 255]));
    }
    for (x, y) in [(410, 22), (20, 2780), (580, 2780)] {
        image.put_pixel(x, y, Rgba([200, 30, 40, 255]));
    }
    image.save(&path).unwrap();
    f.seal();
    let review = f.compare().unwrap();
    let finding = review.results.iter().find(|r| &r.case == case).unwrap();
    assert_eq!(finding.rows, [[0, 2814]]);
    assert_eq!(finding.changed_pixels, Some(2817));
    let doc = f.doc();
    let section = doc
        .split(&format!("id=\"f-{case}\""))
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    let diff = read_png(&f.report.join(case).join("diff.png")).unwrap();
    let windows: Vec<[u32; 4]> = section
        .split("class=\"loupe\"")
        .skip(1)
        .step_by(3)
        .map(|tag| {
            let number = |prefix: &str| {
                tag.split(prefix)
                    .nth(1)
                    .unwrap()
                    .split("px")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap()
            };
            [
                number("left:-"),
                number("top:-"),
                number("width:"),
                number("height:"),
            ]
        })
        .collect();
    assert!(!windows.is_empty());
    for &[x, y, w, h] in &windows {
        assert!(x + w <= 608 && y + h <= 2814);
        assert!(
            (y..y + h).any(|dy| (x..x + w)
                .any(|dx| diff.get_pixel(dx, dy).0 == cargo_gpui_visual::compare::CHANGED)),
            "inspector crop contains no changed pixel: {x},{y},{w},{h}"
        );
        assert!(w > 0 && h > 0 && w <= 304 && h <= 96);
        // Inspector and stage show the same unscaled image coordinates.
        let position = format!("style=\"left:-{x}px;top:-{y}px\"");
        assert_eq!(section.matches(&position).count(), 3);
    }
    assert_eq!(windows, crops(section));
    for (x, y, pixel) in diff.enumerate_pixels() {
        if pixel.0 == cargo_gpui_visual::compare::CHANGED {
            assert!(
                windows
                    .iter()
                    .any(|&[cx, cy, w, h]| (cx..cx + w).contains(&x) && (cy..cy + h).contains(&y)),
                "changed pixel {x},{y} has no inspector crop"
            );
        }
    }
    assert!(section.contains("1 Zeilenband"));
    assert!(section.contains("Detailausschnitte"));
    assert!(!section.contains("1 Bereich"));
    assert_eq!(baseline, baseline_state(&f.base).unwrap());
    // Missing/corrupt/wrong-sized/inconsistent diff data must not invent crops.
    let diff_path = f.report.join(case).join("diff.png");
    let original_diff = fs::read(&diff_path).unwrap();
    for invalid in 0..4 {
        match invalid {
            0 => fs::remove_file(&diff_path).unwrap(),
            1 => fs::write(&diff_path, "invalid PNG").unwrap(),
            2 => RgbaImage::new(1, 1).save(&diff_path).unwrap(),
            _ => RgbaImage::new(608, 2814).save(&diff_path).unwrap(),
        }
        report::write(&f.report, &review).unwrap();
        assert!(f.doc().contains("Detailausschnitte nicht verfügbar"));
        assert!(crops(&f.doc()).is_empty());
        assert!(!f.doc().contains("class=\"loupe\""));
    }
    fs::write(&diff_path, original_diff).unwrap();
    report::write(&f.report, &review).unwrap();
    assert_eq!(baseline, baseline_state(&f.base).unwrap());
    if let Some(output) = std::env::var_os("GPUI_INSPECTOR_FIXTURE_OUTPUT") {
        let output = PathBuf::from(output);
        assert!(!output.exists());
        fs::rename(&f.repo, output).unwrap();
    }
}
