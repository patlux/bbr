//! Concrete forwarding closures for the sidebar [`Callbacks`] slots.
//!
//! GPUI's element listeners require concrete `Fn` closures (they are never
//! `Rc<dyn Fn>`-callable), so the rail wraps each `Rc<dyn Fn>` slot in a
//! small forwarding closure at the composition boundary. The product
//! authority stays in the caller: `bbr-app` fills every slot with the
//! production app methods (re-entering the app entity), and the gallery
//! fills recorder callbacks. This module only adapts the invocation shape —
//! it never invents behavior.

use bbr_domain::{FolderId, SpaceId};
use gpui::{App, KeyDownEvent, MouseDownEvent, MouseUpEvent, ScrollWheelEvent, Window};

use super::model::{
    BoolCallback, FolderCallback, HoverCallback, KeyCallback, MemberCallback, MouseCallback,
    MouseUpCallback, RowCallback, RowUpCallback, SpaceCallback, WheelCallback,
};

/// Forward a `MouseDownEvent` slot as a concrete `Fn` closure.
pub fn mouse(cb: &MouseCallback) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| cb(event, window, cx)
}

/// Forward a `MouseUpEvent` slot as a concrete `Fn` closure.
pub fn mouse_up(cb: &MouseUpCallback) -> impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseUpEvent, window: &mut Window, cx: &mut App| cb(event, window, cx)
}

/// Forward a `KeyDownEvent` slot as a concrete `Fn` closure.
pub fn key(cb: &KeyCallback) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| cb(event, window, cx)
}

/// Forward a `ScrollWheelEvent` slot as a concrete `Fn` closure.
pub fn wheel(cb: &WheelCallback) -> impl Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &ScrollWheelEvent, window: &mut Window, cx: &mut App| cb(event, window, cx)
}

/// Forward a `&bool` slot as a concrete `Fn` closure.
pub fn boolean(cb: &BoolCallback) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |value: &bool, window: &mut Window, cx: &mut App| cb(value, window, cx)
}

/// Forward a row-indexed `MouseDownEvent` slot as a concrete `Fn` closure
/// bound to `index`.
pub fn row(
    cb: &RowCallback,
    index: usize,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| cb(index, event, window, cx)
}

/// Forward a row-indexed `MouseUpEvent` slot as a concrete `Fn` closure
/// bound to `index`.
pub fn row_up(
    cb: &RowUpCallback,
    index: usize,
) -> impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseUpEvent, window: &mut Window, cx: &mut App| cb(index, event, window, cx)
}

/// Forward a row-indexed hover slot as a concrete `Fn` closure bound to
/// `index`.
pub fn hover(cb: &HoverCallback, index: usize) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |hovered: &bool, window: &mut Window, cx: &mut App| cb(index, *hovered, window, cx)
}

/// Forward a folder-id `MouseDownEvent` slot as a concrete `Fn` closure
/// bound to `folder`.
pub fn folder(
    cb: &FolderCallback,
    folder: FolderId,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| cb(folder, event, window, cx)
}

/// Forward a folder-id + member `MouseDownEvent` slot as a concrete `Fn`
/// closure bound to `(folder, member)`.
pub fn member(
    cb: &MemberCallback,
    folder: FolderId,
    member: usize,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        cb(folder, member, event, window, cx)
    }
}

/// Forward a space-id `MouseDownEvent` slot as a concrete `Fn` closure
/// bound to `space`.
pub fn space(
    cb: &SpaceCallback,
    space: SpaceId,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| cb(space, event, window, cx)
}
