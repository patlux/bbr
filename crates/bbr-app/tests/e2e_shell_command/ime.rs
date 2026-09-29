//! Blueprint D02 prerequisite E2E: the command field as a platform
//! text-input client (DESIGN §11.5).
//!
//! Input-method messages go to the browser window's real GPUI root view
//! through its `NSTextInputClient` conformance (`CommandTextInput`), so the
//! AppKit → GPUI → registered input handler path an input method takes is
//! the path under test: a composition shows as marked text, refines, and
//! commits over itself; the client reports its marked and selected ranges
//! in UTF-16 and a candidate-window rect on the field's composition; caret
//! motions extend the selection from its anchor; and once the field stops
//! owning the keyboard no client answers. Typed snapshots under deadlines;
//! no sleeps, no pixel comparisons, no synthetic pointer.

use super::command_portal;
use super::*;
use bbr_testkit::protocol::{CommandInputSnapshot, CommandSurface, Snapshot, TextInputOp};

fn drive(client: &mut Client, op: TextInputOp) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::CommandTextInput { op })
        .expect("text input message");
    client.snapshot().expect("text input snapshot")
}

fn input(snapshot: &Snapshot) -> &CommandInputSnapshot {
    snapshot
        .command_input
        .as_ref()
        .expect("the command field owns the keyboard")
}

/// Draw one real frame (the hidden harness has no display link): the
/// paint registers the input handler and lays the field line out.
fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// Wait until the painted frame registered the field's input handler: the
/// native client answers `selectedRange` with the model's caret.
fn wait_client_ready(client: &mut Client, caret: usize) -> Snapshot {
    let until = deadline();
    loop {
        flush_frame(client);
        let snapshot = drive(client, TextInputOp::Probe);
        let native = snapshot.text_input;
        if native.and_then(|native| native.selected) == Some([caret, caret])
            && snapshot
                .command_input
                .as_ref()
                .is_some_and(|i| i.line.is_some())
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "input client never registered: {native:?}"
        );
    }
}

fn inside(rect: [f32; 4], outer: [f32; 4]) -> bool {
    rect[0] >= outer[0] - 0.5
        && rect[1] >= outer[1] - 0.5
        && rect[0] + rect[2] <= outer[0] + outer[2] + 0.5
        && rect[1] + rect[3] <= outer[1] + outer[3] + 0.5
}

/// The native candidate rect is the input handler's range rect mapped to
/// the screen (AppKit's bottom-left origin) — the input method opens its
/// window on the composition.
fn assert_candidate_rect_on_range(snapshot: &Snapshot) {
    let input = input(snapshot);
    let range = input.range_rect.expect("laid-out range rect");
    let native = snapshot.text_input.expect("native readback");
    let rect = native.first_rect.expect("candidate rect");
    let frame = native.window_frame;
    assert!(
        (rect[0] - (frame[0] + f64::from(range[0]))).abs() < 1.0,
        "{rect:?} vs {range:?} in {frame:?}"
    );
    assert!(
        (rect[2] - f64::from(range[2])).abs() < 1.0,
        "{rect:?} vs {range:?}"
    );
    assert!(
        (rect[3] - f64::from(range[3])).abs() < 1.0,
        "{rect:?} vs {range:?}"
    );
    let top = frame[1] + frame[3] - f64::from(range[1]) - f64::from(range[3]);
    assert!(
        (rect[1] - top).abs() < 1.0,
        "{rect:?} vs {range:?} in {frame:?}"
    );
}

#[test]
#[ignore]
fn the_command_field_composes_through_the_native_text_input_client() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-ime", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // ⌘T: the palette's field owns the keyboard; plain keys still type
        // through the key-down path.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let palette = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Palette,
            deadline(),
        );
        let card = palette.portal.rect.expect("palette rect");
        type_char(&mut client, 'a');
        type_char(&mut client, 'b');
        let ready = wait_client_ready(&mut client, 2);
        assert_eq!(input(&ready).text, "ab");
        assert!(
            inside(input(&ready).line.unwrap(), card),
            "the line lies in the palette"
        );

        // An input method composes "にほ" at the caret: marked text, caret at
        // its end, the candidate window on the composition.
        let composing = drive(
            &mut client,
            TextInputOp::SetMarked {
                text: "にほ".into(),
                selected: [2, 2],
            },
        );
        assert_eq!(composing.command_bar_text, "abにほ");
        let state = input(&composing);
        assert_eq!(state.marked, Some([2, 4]));
        assert_eq!(state.cursor, 4);
        let native = composing.text_input.unwrap();
        assert_eq!(native.marked, Some([2, 4]));
        assert_eq!(native.selected, Some([4, 4]));
        let range = state.range_rect.expect("composition rect");
        let line = state.line.unwrap();
        assert!(
            range[2] > 0.0 && range[0] > line[0],
            "the composition sits after \"ab\": {range:?} in {line:?}"
        );
        assert_candidate_rect_on_range(&composing);

        // It refines, then commits over itself.
        let refined = drive(
            &mut client,
            TextInputOp::SetMarked {
                text: "にほん".into(),
                selected: [3, 3],
            },
        );
        assert_eq!(refined.command_bar_text, "abにほん");
        assert_eq!(input(&refined).marked, Some([2, 5]));
        let committed = drive(
            &mut client,
            TextInputOp::Insert {
                text: "日本".into(),
            },
        );
        assert_eq!(committed.command_bar_text, "ab日本");
        let state = input(&committed);
        assert_eq!(state.marked, None);
        assert_eq!(state.cursor, 4);
        assert_eq!(committed.text_input.unwrap().marked, None);
        assert_eq!(committed.text_input.unwrap().selected, Some([4, 4]));
        // The model keeps UTF-8 bytes; the protocol view is UTF-16.
        assert_eq!(committed.command_cursor, "ab日本".len());

        // ⇧← twice selects "日本" from the anchor; typing replaces it.
        key(&mut client, "shift-left");
        key(&mut client, "shift-left");
        let selected = drive(&mut client, TextInputOp::Probe);
        assert_eq!(input(&selected).selection, Some([2, 4]));
        assert_eq!(selected.text_input.unwrap().selected, Some([2, 4]));
        assert_candidate_rect_on_range(&selected);
        type_char(&mut client, 'x');
        let replaced = client.snapshot().expect("replaced");
        assert_eq!(replaced.command_bar_text, "abx");

        // `unmarkText` keeps a composition as typed text.
        drive(
            &mut client,
            TextInputOp::SetMarked {
                text: "か".into(),
                selected: [1, 1],
            },
        );
        let kept = drive(&mut client, TextInputOp::Unmark);
        assert_eq!(kept.command_bar_text, "abxか");
        assert_eq!(input(&kept).marked, None);

        // ⌘← jumps to the start; ⇧⌘→ selects to the end.
        key(&mut client, "cmd-left");
        key(&mut client, "shift-cmd-right");
        let all = client.snapshot().expect("line selection");
        assert_eq!(input(&all).selection, Some([0, 4]));

        // Escape: the field gives up the keyboard and no client answers.
        key(&mut client, "escape");
        since = wait_chrome_focus(&mut client, false, since);
        flush_frame(&mut client);
        let closed = drive(&mut client, TextInputOp::Probe);
        assert!(closed.command_input.is_none());
        assert_eq!(
            closed.text_input.unwrap().selected,
            None,
            "no input handler is registered once the field lets go"
        );
        assert_eq!(closed.command_surface, CommandSurface::None);

        // ⌘L: the overlay opens with the URL selected; a composition
        // replaces the selection, like typing.
        let overlay = cmd_l(&mut client, &mut since);
        let len = page_one.encode_utf16().count();
        let ready = wait_client_ready_selected(&mut client, [0, len]);
        assert!(inside(
            input(&ready).line.unwrap(),
            overlay.portal.rect.unwrap()
        ));
        let composing = drive(
            &mut client,
            TextInputOp::SetMarked {
                text: "え".into(),
                selected: [1, 1],
            },
        );
        assert_eq!(composing.command_bar_text, "え");
        assert_eq!(input(&composing).marked, Some([0, 1]));
        assert_candidate_rect_on_range(&composing);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

fn wait_client_ready_selected(client: &mut Client, selection: [usize; 2]) -> Snapshot {
    let until = deadline();
    loop {
        flush_frame(client);
        let snapshot = drive(client, TextInputOp::Probe);
        let native = snapshot.text_input;
        if native.and_then(|native| native.selected) == Some(selection)
            && snapshot
                .command_input
                .as_ref()
                .is_some_and(|i| i.line.is_some())
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "input client never registered: {native:?}"
        );
    }
}
