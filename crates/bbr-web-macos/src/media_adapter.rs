//! Blueprint D24 (board S-8, packet `media-session-capability-contract`):
//! the media adapter behind the sidebar media card.
//!
//! The public per-web-view WebKit API proves only a playback state and
//! Pause, so the adapter reads the page's media through BBR's own client
//! content world — never the page's world: the probe sees the shared DOM,
//! page scripts can neither see nor collide with it. Each probe picks the
//! page's current `<audio>`/`<video>` element (the playing one, else the
//! one it controlled last, else one that has played), remembers it in the
//! client world, and reports playback, elapsed/duration (none for live or
//! unknown lengths), mute and the Media Session title. The capabilities are
//! what the element proves: play, pause and mute, seek only with a known
//! duration; Previous/Next are never claimed (the page's Media Session
//! action handlers live in the page's world). An observation is emitted
//! only when it changes. Commands act on the remembered element.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, Message};
use objc2_foundation::{NSError, NSString};
use objc2_web_kit::{WKContentWorld, WKMediaPlaybackState, WKWebView};

use bbr_web::media::{
    MediaCapabilities, MediaCommand, MediaMetadata, MediaObservation, MediaPlayback, MediaTiming,
};
use bbr_web::{WebEvent, WebViewId};

use crate::delegate::EventSink;

/// The probe, run in BBR's client content world.
pub(crate) const PROBE_SCRIPT: &str = r#"(() => {
  const all = Array.from(document.querySelectorAll('audio,video'));
  const kept = (window.__bbrMedia && all.includes(window.__bbrMedia)) ? window.__bbrMedia : null;
  const el = all.find(e => !e.paused && !e.ended)
    || kept
    || all.find(e => e.currentTime > 0 && !e.ended);
  if (!el) { window.__bbrMedia = null; return JSON.stringify({ present: false }); }
  window.__bbrMedia = el;
  let title = null, artist = null;
  try {
    const meta = navigator.mediaSession && navigator.mediaSession.metadata;
    if (meta) { title = meta.title || null; artist = meta.artist || null; }
  } catch (e) {}
  const d = el.duration;
  return JSON.stringify({
    present: true,
    paused: el.paused,
    ended: el.ended,
    elapsed: el.currentTime,
    duration: (isFinite(d) && d > 0) ? d : null,
    live: d === Infinity,
    muted: el.muted,
    title: title || document.title || null,
    artist,
  });
})()"#;

/// The command script for one command.
pub(crate) fn command_script(command: MediaCommand) -> Option<String> {
    let body = match command {
        MediaCommand::Play => "const p = el.play(); if (p && p.catch) p.catch(() => {});".into(),
        MediaCommand::Pause => "el.pause();".into(),
        MediaCommand::Seek { position } if position.is_finite() => {
            format!("el.currentTime = {position};")
        }
        MediaCommand::Mute => "el.muted = true;".into(),
        MediaCommand::Unmute => "el.muted = false;".into(),
        _ => return None,
    };
    Some(format!(
        "(() => {{ const el = window.__bbrMedia; if (!el) return false; {body} return true; }})()"
    ))
}

/// The observation a probe payload reports.
pub(crate) fn observation_from(payload: &str) -> Option<MediaObservation> {
    let probe: serde_json::Value = serde_json::from_str(payload).ok()?;
    let number = |key: &str| probe.get(key).and_then(serde_json::Value::as_f64);
    let flag = |key: &str| probe.get(key).and_then(serde_json::Value::as_bool);
    let text = |key: &str| {
        probe
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
    };
    if !flag("present")? || flag("ended").unwrap_or(false) {
        return Some(MediaObservation::native(MediaPlayback::None));
    }
    let paused = flag("paused").unwrap_or(true);
    let muted = flag("muted");
    let duration = number("duration").filter(|d| d.is_finite() && *d > 0.0);
    Some(MediaObservation {
        playback: if paused {
            MediaPlayback::Paused
        } else {
            MediaPlayback::Playing
        },
        capabilities: MediaCapabilities {
            play: true,
            pause: true,
            seek: duration.is_some(),
            previous: false,
            next: false,
            mute: muted.is_some(),
        },
        timing: MediaTiming {
            // Whole seconds: the card shows m:ss, and sub-second drift would
            // re-report every probe for nothing.
            elapsed: number("elapsed").map(f64::floor),
            duration: duration.map(f64::floor),
            live: flag("live").unwrap_or(false),
        },
        metadata: MediaMetadata {
            title: text("title"),
            subtitle: text("artist"),
        },
        muted,
    })
}

/// The media probes of every live view (shares the engine's event sink).
pub(crate) struct MediaHost {
    sink: EventSink,
    /// Views whose last probe found media (they report `None` once when it
    /// goes away).
    present: Rc<RefCell<HashSet<WebViewId>>>,
}

impl MediaHost {
    pub(crate) fn new(sink: EventSink) -> Self {
        Self {
            sink,
            present: Rc::default(),
        }
    }

    /// Probe one view. WebKit's own playback state gates the page probe:
    /// a view WebKit reports no media for (and that had none) costs one
    /// native query and no script.
    pub(crate) fn probe(&self, mtm: MainThreadMarker, view: WebViewId, web_view: &WKWebView) {
        let sink = self.sink.clone();
        let present = self.present.clone();
        let target: Retained<WKWebView> = web_view.retain();
        let native = RcBlock::new(move |state: WKMediaPlaybackState| {
            let had = present.borrow().contains(&view);
            if state == WKMediaPlaybackState::None && !had {
                return;
            }
            let sink = sink.clone();
            let present = present.clone();
            let completion = RcBlock::new(move |result: *mut AnyObject, error: *mut NSError| {
                if !error.is_null() {
                    return;
                }
                // SAFETY: the JS result (a string for this script) or null.
                let Some(payload) = (unsafe { result.as_ref() })
                    .and_then(|object| object.downcast_ref::<NSString>())
                    .map(|s| s.to_string())
                else {
                    return;
                };
                let Some(mut observation) = observation_from(&payload) else {
                    return;
                };
                if state == WKMediaPlaybackState::Suspended
                    && observation.playback != MediaPlayback::None
                {
                    observation.playback = MediaPlayback::Suspended;
                }
                let gone = observation.playback == MediaPlayback::None;
                let report = {
                    let mut present = present.borrow_mut();
                    if gone {
                        present.remove(&view)
                    } else {
                        present.insert(view);
                        true
                    }
                };
                if report {
                    if let Some(sink) = sink.borrow_mut().as_mut() {
                        sink(WebEvent::MediaObserved { view, observation });
                    }
                }
            });
            evaluate(mtm, &target, PROBE_SCRIPT, &completion);
        });
        // SAFETY: main thread; a live view; WebKit copies the block.
        unsafe { web_view.requestMediaPlaybackStateWithCompletionHandler(&native) };
    }

    /// Apply a command to the view's remembered element.
    pub(crate) fn command(
        &self,
        mtm: MainThreadMarker,
        web_view: &WKWebView,
        command: MediaCommand,
    ) {
        let Some(script) = command_script(command) else {
            return;
        };
        let completion = RcBlock::new(|_result: *mut AnyObject, _error: *mut NSError| {});
        evaluate(mtm, web_view, &script, &completion);
    }

    /// Forget a view that went away.
    pub(crate) fn forget(&self, view: WebViewId) {
        self.present.borrow_mut().remove(&view);
    }
}

fn evaluate(
    mtm: MainThreadMarker,
    web_view: &WKWebView,
    script: &str,
    completion: &RcBlock<dyn Fn(*mut AnyObject, *mut NSError)>,
) {
    let script = NSString::from_str(script);
    // SAFETY: the process-wide default client world (retained by WebKit).
    let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
    // SAFETY: main thread; a live view; WebKit copies the block.
    unsafe {
        web_view.evaluateJavaScript_inFrame_inContentWorld_completionHandler(
            &script,
            None,
            &world,
            Some(completion),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes_report_what_the_element_proves() {
        let playing = observation_from(
            r#"{"present":true,"paused":false,"ended":false,"elapsed":12.7,"duration":200.4,"muted":true,"title":"Song","artist":"Band"}"#,
        )
        .unwrap();
        assert_eq!(playing.playback, MediaPlayback::Playing);
        assert!(playing.capabilities.seek && playing.capabilities.mute);
        assert!(!playing.capabilities.previous && !playing.capabilities.next);
        assert_eq!(playing.timing.elapsed, Some(12.0));
        assert_eq!(playing.timing.duration, Some(200.0));
        assert_eq!(playing.metadata.title.as_deref(), Some("Song"));
        assert_eq!(playing.metadata.subtitle.as_deref(), Some("Band"));
        let live = observation_from(
            r#"{"present":true,"paused":true,"ended":false,"elapsed":3,"duration":null,"muted":false,"title":null}"#,
        )
        .unwrap();
        assert_eq!(live.playback, MediaPlayback::Paused);
        assert!(!live.capabilities.seek, "no seek without a known length");
        assert_eq!(live.timing.duration, None);
        assert!(!live.timing.live);
        let stream = observation_from(
            r#"{"present":true,"paused":false,"ended":false,"elapsed":3,"duration":null,"live":true,"muted":false}"#,
        )
        .unwrap();
        assert!(stream.timing.live && !stream.capabilities.seek);
        let gone = observation_from(r#"{"present":false}"#).unwrap();
        assert_eq!(gone.playback, MediaPlayback::None);
        assert!(observation_from("nonsense").is_none());
    }

    #[test]
    fn commands_act_on_the_remembered_element_only() {
        assert!(command_script(MediaCommand::Pause)
            .unwrap()
            .contains("el.pause()"));
        assert!(command_script(MediaCommand::Seek { position: 30.0 })
            .unwrap()
            .contains("el.currentTime = 30"));
        assert!(command_script(MediaCommand::Seek { position: f64::NAN }).is_none());
        assert!(command_script(MediaCommand::Next).is_none());
    }
}
