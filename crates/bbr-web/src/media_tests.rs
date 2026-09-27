use super::*;

fn view(id: u64) -> WebViewId {
    WebViewId(id)
}

fn observation(playback: MediaPlayback, capabilities: MediaCapabilities) -> MediaObservation {
    MediaObservation {
        playback,
        capabilities,
        timing: MediaTiming {
            elapsed: Some(12.0),
            duration: Some(300.0),
        },
        metadata: MediaMetadata {
            title: Some("Track".into()),
            subtitle: Some("example.com".into()),
        },
        muted: Some(false),
    }
}

fn all() -> MediaCapabilities {
    MediaCapabilities {
        play: true,
        pause: true,
        seek: true,
        previous: true,
        next: true,
        mute: true,
    }
}

fn started(change: Option<MediaSessionChange>) -> MediaSession {
    match change {
        Some(MediaSessionChange::Started(session)) => session,
        other => panic!("expected a started session, got {other:?}"),
    }
}

const COMMANDS: [MediaCommand; 7] = [
    MediaCommand::Play,
    MediaCommand::Pause,
    MediaCommand::Seek { position: 30.0 },
    MediaCommand::Previous,
    MediaCommand::Next,
    MediaCommand::Mute,
    MediaCommand::Unmute,
];

#[test]
fn every_capability_subset_authorizes_exactly_its_commands() {
    // All 64 subsets of {play, pause, seek, previous, next, mute}: a command
    // is authorized iff its bit is set; otherwise it is a typed refusal.
    for bits in 0u8..64 {
        let capabilities = MediaCapabilities {
            play: bits & 1 != 0,
            pause: bits & 2 != 0,
            seek: bits & 4 != 0,
            previous: bits & 8 != 0,
            next: bits & 16 != 0,
            mute: bits & 32 != 0,
        };
        let mut registry = MediaSessionRegistry::new();
        let session =
            started(registry.observe(view(1), observation(MediaPlayback::Playing, capabilities)));
        for command in COMMANDS {
            let result = registry.authorize(session.id, &command);
            if capabilities.allows(&command) {
                assert!(result.is_ok(), "{bits:06b} {command:?}");
            } else {
                assert_eq!(
                    result.err(),
                    Some(MediaCommandOutcome::Unsupported),
                    "{bits:06b} {command:?}"
                );
            }
        }
    }
}

#[test]
fn the_native_web_view_source_proves_pause_only() {
    let mut registry = MediaSessionRegistry::new();
    let session =
        started(registry.observe(view(1), MediaObservation::native(MediaPlayback::Playing)));
    assert!(registry.authorize(session.id, &MediaCommand::Pause).is_ok());
    for command in COMMANDS
        .iter()
        .filter(|c| !matches!(c, MediaCommand::Pause))
    {
        assert_eq!(
            registry.authorize(session.id, command).err(),
            Some(MediaCommandOutcome::Unsupported),
            "{command:?}"
        );
    }
    assert_eq!(session.timing, MediaTiming::default(), "no fake timing");
    assert_eq!(session.muted, None, "no fake mute state");
}

#[test]
fn replaced_sessions_reject_old_generations_without_touching_the_new_one() {
    let mut registry = MediaSessionRegistry::new();
    let first = started(registry.observe(view(1), observation(MediaPlayback::Playing, all())));
    // Navigation / reload / crash / close / WebView replacement invalidate.
    assert_eq!(
        registry.invalidate(view(1)),
        Some(MediaSessionChange::Ended(first.id))
    );
    assert_eq!(
        registry.authorize(first.id, &MediaCommand::Pause).err(),
        Some(MediaCommandOutcome::Ended)
    );
    // The same tab plays again: a new generation; the old id is stale and
    // can never reach the new session.
    let second = started(registry.observe(view(1), observation(MediaPlayback::Playing, all())));
    assert_ne!(first.id.generation, second.id.generation);
    assert_eq!(
        registry.authorize(first.id, &MediaCommand::Pause).err(),
        Some(MediaCommandOutcome::Stale)
    );
    assert!(registry.authorize(second.id, &MediaCommand::Pause).is_ok());
}

#[test]
fn ended_playback_ends_the_session() {
    let mut registry = MediaSessionRegistry::new();
    let session = started(registry.observe(view(3), observation(MediaPlayback::Paused, all())));
    assert_eq!(
        registry.observe(view(3), observation(MediaPlayback::None, all())),
        Some(MediaSessionChange::Ended(session.id))
    );
    assert!(registry.session(view(3)).is_none());
    assert_eq!(
        registry.observe(view(3), observation(MediaPlayback::None, all())),
        None,
        "no session, nothing to end"
    );
}

#[test]
fn two_sessions_stay_independent_and_order_by_recency() {
    let mut registry = MediaSessionRegistry::new();
    let a = started(registry.observe(view(1), observation(MediaPlayback::Playing, all())));
    let b = started(registry.observe(view(2), observation(MediaPlayback::Paused, all())));
    assert_eq!(registry.ordered()[0].id, b.id, "the newest session first");
    // A pausing session keeps its place; a session that starts playing
    // again moves to the front.
    registry.observe(view(1), observation(MediaPlayback::Paused, all()));
    assert_eq!(registry.ordered()[0].id, b.id);
    registry.observe(view(1), observation(MediaPlayback::Playing, all()));
    let ordered: Vec<_> = registry.ordered().iter().map(|s| s.id).collect();
    assert_eq!(ordered, vec![a.id, b.id]);
    // Ending one session never touches the other.
    registry.invalidate(view(1));
    assert_eq!(registry.session(view(2)).map(|s| s.id), Some(b.id));
    assert!(registry.authorize(b.id, &MediaCommand::Play).is_ok());
    assert_eq!(
        registry.authorize(a.id, &MediaCommand::Play).err(),
        Some(MediaCommandOutcome::Ended)
    );
}

#[test]
fn unknown_or_live_duration_has_no_seek_and_no_fake_total() {
    for duration in [None, Some(f64::INFINITY), Some(f64::NAN), Some(0.0)] {
        let mut registry = MediaSessionRegistry::new();
        let mut live = observation(MediaPlayback::Playing, all());
        live.timing.duration = duration;
        let session = started(registry.observe(view(1), live));
        assert!(!session.capabilities.seek, "{duration:?}");
        assert_eq!(
            registry
                .authorize(session.id, &MediaCommand::Seek { position: 1.0 })
                .err(),
            Some(MediaCommandOutcome::Unsupported)
        );
    }
}

#[test]
fn seek_outside_the_known_duration_and_play_while_suspended_are_refused() {
    let mut registry = MediaSessionRegistry::new();
    let session = started(registry.observe(view(1), observation(MediaPlayback::Playing, all())));
    for position in [-1.0, 300.5, f64::NAN] {
        assert_eq!(
            registry
                .authorize(session.id, &MediaCommand::Seek { position })
                .err(),
            Some(MediaCommandOutcome::NotControllable),
            "{position}"
        );
    }
    assert!(registry
        .authorize(session.id, &MediaCommand::Seek { position: 300.0 })
        .is_ok());
    registry.observe(view(1), observation(MediaPlayback::Suspended, all()));
    assert_eq!(
        registry.authorize(session.id, &MediaCommand::Play).err(),
        Some(MediaCommandOutcome::NotControllable),
        "a host suspension is not lifted by the page's Play"
    );
    assert!(registry.authorize(session.id, &MediaCommand::Pause).is_ok());
}

#[test]
fn mute_needs_a_readable_mute_state() {
    let mut registry = MediaSessionRegistry::new();
    let mut observed = observation(MediaPlayback::Playing, all());
    observed.muted = None;
    let session = started(registry.observe(view(1), observed));
    assert!(!session.capabilities.mute);
    assert_eq!(
        registry.authorize(session.id, &MediaCommand::Mute).err(),
        Some(MediaCommandOutcome::Unsupported)
    );
}

#[test]
fn unchanged_observations_emit_nothing() {
    let mut registry = MediaSessionRegistry::new();
    started(registry.observe(view(1), observation(MediaPlayback::Playing, all())));
    assert_eq!(
        registry.observe(view(1), observation(MediaPlayback::Playing, all())),
        None
    );
    assert!(matches!(
        registry.observe(view(1), observation(MediaPlayback::Paused, all())),
        Some(MediaSessionChange::Updated(_))
    ));
}
