//! The favicon disk cache (task packet `favicon-disk-cache`).

use super::*;

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bbr-favicon-cache-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn records(items: &[(&str, &[u8])]) -> Records {
    items
        .iter()
        .map(|(host, bytes)| (host.to_string(), bytes.to_vec()))
        .collect()
}

#[test]
fn records_round_trip_per_profile_in_order() {
    let root = dir("round-trip");
    let (mut disk, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(1), ProfileId(2)]);
    assert!(loaded.is_empty(), "nothing kept yet");
    let one = records(&[("b.example", b"\x89PNG-b"), ("a.example:8080", b"ico")]);
    let two = records(&[("a.example", b"other profile")]);
    disk.sync(&[(ProfileId(1), one.clone()), (ProfileId(2), two.clone())]);

    let (_, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(1), ProfileId(2)]);
    assert_eq!(loaded, vec![(ProfileId(1), one), (ProfileId(2), two)]);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn a_change_writes_the_diff_and_an_eviction_deletes_its_file() {
    let root = dir("diff");
    let (mut disk, _) = FaviconDisk::open(root.clone(), &[ProfileId(1)]);
    disk.sync(&[(
        ProfileId(1),
        records(&[("a.example", b"a1"), ("b.example", b"b1")]),
    )]);
    let profile = root.join("1");
    let a = profile.join(file_name("a.example"));
    let b = profile.join(file_name("b.example"));
    assert!(a.exists() && b.exists());

    // b replaced, a evicted, c added.
    let now = records(&[("b.example", b"b2"), ("c.example", b"c1")]);
    disk.sync(&[(ProfileId(1), now.clone())]);
    assert!(!a.exists(), "the evicted record's file is gone");
    assert_eq!(std::fs::read(&b).unwrap(), b"b2");
    let (_, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(1)]);
    assert_eq!(loaded, vec![(ProfileId(1), now)]);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn a_forgotten_profile_and_an_unknown_one_lose_their_directory() {
    let root = dir("forget");
    let (mut disk, _) = FaviconDisk::open(root.clone(), &[ProfileId(1), ProfileId(2)]);
    disk.sync(&[
        (ProfileId(1), records(&[("a.example", b"a")])),
        (ProfileId(2), records(&[("b.example", b"b")])),
    ]);
    // Profile 1's records were cleared: its directory goes.
    disk.sync(&[(ProfileId(2), records(&[("b.example", b"b")]))]);
    assert!(!root.join("1").exists());
    assert!(root.join("2").exists());

    // Profile 2 was deleted while the app did not run: removed at launch.
    let (_, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(3)]);
    assert!(loaded.is_empty());
    assert!(!root.join("2").exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unreadable_and_oversized_records_are_skipped() {
    let root = dir("damaged");
    let profile = root.join("1");
    std::fs::create_dir_all(&profile).unwrap();
    let good = file_name("good.example");
    let big = file_name("big.example");
    std::fs::write(profile.join(&good), b"ok").unwrap();
    std::fs::write(profile.join(&big), vec![1; MAX_RECORD_BYTES + 1]).unwrap();
    std::fs::write(
        profile.join(INDEX),
        format!("1\nzz\n{big}\n{}\n{good}", file_name("missing.example")),
    )
    .unwrap();
    let (_, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(1)]);
    assert_eq!(
        loaded,
        vec![(ProfileId(1), records(&[("good.example", b"ok")]))]
    );

    // An index of another format version is not read.
    std::fs::write(profile.join(INDEX), format!("2\n{good}")).unwrap();
    let (_, loaded) = FaviconDisk::open(root.clone(), &[ProfileId(1)]);
    assert!(loaded.is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn host_file_names_round_trip() {
    for host in ["example.com", "localhost", "xn--bcher-kva.example"] {
        assert_eq!(host_of(&file_name(host)).as_deref(), Some(host));
    }
    assert_eq!(host_of("abc"), None);
    assert_eq!(host_of("zz"), None);
    assert_eq!(host_of(""), None);
}
