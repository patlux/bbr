//! Deterministic renderer-isolation fixtures.
//!
//! These pages exercise renderer behavior only. The HTTP server stays
//! responsive and records one normal hit per request.

pub(crate) const BUSY_PATH: &str = "/__fx/hang/busy";
pub(crate) const HEAVY_PATH: &str = "/__fx/hang/heavy";
pub(crate) const DIALOG_PATH: &str = "/__fx/hang/dialog";
pub(crate) const BEFOREUNLOAD_PATH: &str = "/__fx/hang/beforeunload";

pub(crate) fn body(path: &str) -> Option<Vec<u8>> {
    let html = match path {
        BUSY_PATH => BUSY,
        HEAVY_PATH => HEAVY,
        DIALOG_PATH => DIALOG,
        BEFOREUNLOAD_PATH => BEFOREUNLOAD,
        _ => return None,
    };
    Some(html.as_bytes().to_vec())
}

const BUSY: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>BBR Hang Busy</title></head>
<body><main><h1>Busy renderer</h1><p id="state">ready</p></main>
<script>
  document.getElementById("state").textContent = "busy";
  window.__bbrHangReady = true;
  for (;;) {}
</script></body></html>"#;

const HEAVY: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>BBR Hang Heavy</title></head>
<body><main><h1>Heavy renderer</h1><output id="state">ready:0</output></main>
<script>
  (() => {
    const WORK_UNITS = 500000;
    const state = { turns: 0, units: 0, checksum: 0, stopped: false };
    window.__bbrHeavyState = state;
    window.__bbrHeavyStop = () => { state.stopped = true; };
    const run = () => {
      if (state.stopped) {
        document.getElementById("state").textContent = `stopped:${state.turns}:${state.units}:${state.checksum}`;
        return;
      }
      let checksum = state.checksum;
      for (let i = 0; i < WORK_UNITS; i += 1) {
        checksum = (checksum + ((i ^ state.turns) & 255)) >>> 0;
      }
      state.turns += 1;
      state.units += WORK_UNITS;
      state.checksum = checksum;
      document.getElementById("state").textContent = `running:${state.turns}:${state.units}:${state.checksum}`;
      setTimeout(run, 0);
    };
    setTimeout(run, 0);
  })();
</script></body></html>"#;

const DIALOG: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>BBR Hang Dialog</title></head>
<body><main><h1>Dialog fixture</h1><output id="state">ready</output></main>
<script>
  window.__bbrDialogReady = true;
  window.__bbrRunDialogs = () => {
    const alertResult = alert("bbr-alert");
    const confirmResult = confirm("bbr-confirm");
    const promptResult = prompt("bbr-prompt", "bbr-default");
    const result = { alertResult, confirmResult, promptResult };
    window.__bbrDialogResult = result;
    document.getElementById("state").textContent = JSON.stringify(result);
    return result;
  };
  setTimeout(() => window.__bbrRunDialogs(), 0);
</script></body></html>"#;

const BEFOREUNLOAD: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>BBR Hang Beforeunload</title></head>
<body><main><h1>Beforeunload fixture</h1><output id="state">registered</output></main>
<script>
  window.__bbrBeforeUnloadCount = 0;
  window.addEventListener("beforeunload", event => {
    window.__bbrBeforeUnloadCount += 1;
    event.preventDefault();
    event.returnValue = "";
  });
  window.__bbrBeforeUnloadReady = true;
</script></body></html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_routes_are_distinct_and_content_addressed() {
        let paths = [BUSY_PATH, HEAVY_PATH, DIALOG_PATH, BEFOREUNLOAD_PATH];
        for path in paths {
            assert!(body(path).is_some(), "missing hang fixture {path}");
        }
        assert!(body("/__fx/hang/nope").is_none());
    }

    #[test]
    fn busy_is_synchronous_and_never_yields() {
        let page = String::from_utf8(body(BUSY_PATH).unwrap()).unwrap();
        assert!(page.contains("for (;;) {}"));
        assert!(!page.contains("setTimeout(run"));
    }

    #[test]
    fn heavy_yields_and_exposes_deterministic_state() {
        let page = String::from_utf8(body(HEAVY_PATH).unwrap()).unwrap();
        assert!(page.contains("window.__bbrHeavyState"));
        assert!(page.contains("window.__bbrHeavyStop"));
        assert!(page.contains("const WORK_UNITS = 500000"));
        assert!(page.contains("setTimeout(run, 0)"));
        assert!(page.contains("state.turns += 1"));
        assert!(page.contains("state.units += WORK_UNITS"));
    }

    #[test]
    fn dialog_and_beforeunload_expose_readiness() {
        let dialog = String::from_utf8(body(DIALOG_PATH).unwrap()).unwrap();
        assert!(dialog.contains("alert(\"bbr-alert\")"));
        assert!(dialog.contains("confirm(\"bbr-confirm\")"));
        assert!(dialog.contains("prompt(\"bbr-prompt\""));
        assert!(dialog.contains("window.__bbrDialogReady = true"));
        assert!(dialog.contains("setTimeout(() => window.__bbrRunDialogs(), 0)"));

        let before = String::from_utf8(body(BEFOREUNLOAD_PATH).unwrap()).unwrap();
        assert!(before.contains("addEventListener(\"beforeunload\""));
        assert!(before.contains("window.__bbrBeforeUnloadReady = true"));
    }
}
