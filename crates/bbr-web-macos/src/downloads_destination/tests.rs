//! Contract tests for the pure destination planning rules (no display
//! server, no filesystem).

use bbr_web::downloads::{DestinationDecision, StartRequest};

use super::{destination_plan, uniquify_name, url_filename, DestinationPlan};

fn id(n: u64) -> bbr_web::downloads::DownloadId {
    bbr_web::downloads::DownloadId(n)
}

fn suggested_start(url: &str, name: &str) -> StartRequest {
    StartRequest {
        id: id(1),
        url: url.into(),
        destination: DestinationDecision::Suggested(name.into()),
        retry_of: None,
    }
}

#[test]
fn explicit_destination_is_verbatim() {
    let request = StartRequest {
        id: id(1),
        url: "https://example.com/report.pdf".into(),
        destination: DestinationDecision::Explicit("/tmp/saved.pdf".into()),
        retry_of: None,
    };
    assert_eq!(
        destination_plan(&request),
        DestinationPlan::Verbatim("/tmp/saved.pdf".into())
    );
}

#[test]
fn suggested_destination_keeps_the_name() {
    assert_eq!(
        destination_plan(&suggested_start(
            "https://example.com/a/report.pdf",
            "report.pdf"
        )),
        DestinationPlan::Suggest {
            filename: "report.pdf".into()
        }
    );
}

#[test]
fn empty_suggestion_derives_the_name_from_the_url() {
    assert_eq!(
        destination_plan(&suggested_start("https://example.com/a/report.pdf", "")),
        DestinationPlan::Suggest {
            filename: "report.pdf".into()
        }
    );
}

#[test]
fn url_filename_strips_query_fragment_and_directory() {
    assert_eq!(
        url_filename("https://example.com/a/b/file.tar.gz?x=1"),
        "file.tar.gz"
    );
    assert_eq!(
        url_filename("https://example.com/file.pdf#frag"),
        "file.pdf"
    );
    assert_eq!(url_filename("https://example.com/"), "download");
    assert_eq!(url_filename("https://example.com"), "download");
    assert_eq!(url_filename("file.pdf"), "file.pdf");
}

#[test]
fn uniquify_name_appends_a_counter_before_the_extension() {
    let existing = ["report.pdf", "report 2.pdf", "report 3.pdf"];
    let exists = |name: &str| existing.contains(&name);
    assert_eq!(uniquify_name("report.pdf", &exists), "report 4.pdf");
    // First free name wins.
    let existing2 = ["report.pdf"];
    let exists2 = |name: &str| existing2.contains(&name);
    assert_eq!(uniquify_name("report.pdf", &exists2), "report 2.pdf");
}

#[test]
fn uniquify_name_handles_extensionless_and_hidden_names() {
    let exists = |name: &str| name == "README";
    assert_eq!(uniquify_name("README", &exists), "README 2");
    let hidden = |name: &str| name == ".env";
    assert_eq!(uniquify_name(".env", &hidden), ".env 2");
}

#[test]
fn uniquify_name_free_name_is_unchanged() {
    let exists = |_: &str| false;
    assert_eq!(uniquify_name("report.pdf", &exists), "report.pdf");
}
