//! Blueprint D19 (several windows, task packet `multi-window.md`): the
//! handles through which every browser window's shell reaches shared state.
//!
//! Each browser window has its own `BbrApp` entity (its UI state). State
//! that belongs to the browser rather than to one window — the controller,
//! the store, downloads, permissions, media, … — sits behind a [`Shared`]
//! handle that every window clones. `shared.read()` borrows it for reading
//! and `shared.get()` for changing it; each guard lives to the end of its
//! statement (or through an `if let`/`match` whose scrutinee made it).
//! Any number of reads may overlap, but a statement that changes a handle
//! must not read or change it again, directly or through a method it
//! calls — the second borrow panics with the type's name. Clippy's
//! `significant_drop_in_scrutinee` finds guards kept alive by a scrutinee.
//!
//! A mutable access through a guard bumps the app-wide shared generation;
//! the other windows compare it to what they last rendered and re-render
//! when it moved.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use bbr_browser::{BrowserController, WindowKey};

thread_local! {
    /// Bumped by every mutable access to shared state (GPUI runs the app
    /// on the main thread only).
    static GENERATION: Cell<u64> = const { Cell::new(0) };
}

/// The current shared generation (moves whenever a window changed shared
/// state the other windows draw).
pub(crate) fn generation() -> u64 {
    GENERATION.with(Cell::get)
}

/// Record a change the other windows must draw (after a bookkeeping
/// access found one).
pub(crate) fn bump() {
    GENERATION.with(|generation| generation.set(generation.get().wrapping_add(1)));
}

/// A handle to state every browser window shares (see the module docs).
pub(crate) struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    /// Borrow the state for reading (overlapping reads are fine).
    pub(crate) fn read(&self) -> SharedRef<'_, T> {
        SharedRef(self.0.try_borrow().unwrap_or_else(|_| {
            panic!(
                "shared {} read while a statement changes it",
                std::any::type_name::<T>()
            )
        }))
    }

    /// Borrow the state for changing it (exclusive for the statement).
    pub(crate) fn get(&self) -> SharedMut<'_, T> {
        SharedMut {
            guard: self.borrow_raw(),
            mutated: false,
            quiet: false,
        }
    }

    /// Change bookkeeping the other windows do not draw (cadence counters,
    /// idempotent re-assertions): no re-render is asked for. Call
    /// [`bump`] when the change turns out to be visible.
    pub(crate) fn bookkeep(&self) -> SharedMut<'_, T> {
        SharedMut {
            guard: self.borrow_raw(),
            mutated: false,
            quiet: true,
        }
    }

    /// Replace the state (a mutation).
    pub(crate) fn set(&self, value: T) -> T {
        std::mem::replace(&mut *self.get(), value)
    }

    fn borrow_raw(&self) -> RefMut<'_, T> {
        self.0.try_borrow_mut().unwrap_or_else(|_| {
            panic!(
                "shared {} changed while borrowed (one change per statement)",
                std::any::type_name::<T>()
            )
        })
    }
}

impl<T: Copy> Shared<T> {
    /// The value (a read).
    pub(crate) fn value(&self) -> T {
        *self.read()
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.try_borrow() {
            Ok(value) => value.fmt(f),
            Err(_) => f.write_str("Shared(<borrowed>)"),
        }
    }
}

/// A read of shared state.
#[clippy::has_significant_drop]
pub(crate) struct SharedRef<'a, T>(Ref<'a, T>);

impl<T> Deref for SharedRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

/// A change of shared state.
#[clippy::has_significant_drop]
pub(crate) struct SharedMut<'a, T> {
    guard: RefMut<'a, T>,
    mutated: bool,
    quiet: bool,
}

impl<T> Deref for SharedMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> DerefMut for SharedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.mutated = true;
        &mut self.guard
    }
}

impl<T> Drop for SharedMut<'_, T> {
    fn drop(&mut self) {
        if self.mutated && !self.quiet {
            bump();
        }
    }
}

/// A window's access to the shared controller: every borrow first switches
/// the controller to this window's context, so the window is explicit at
/// every access.
#[derive(Clone)]
pub(crate) struct ControllerHandle {
    shared: Shared<BrowserController>,
    window: WindowKey,
}

impl ControllerHandle {
    /// The first window's handle.
    pub(crate) fn new(controller: BrowserController) -> Self {
        let window = controller.current_window();
        Self {
            shared: Shared::new(controller),
            window,
        }
    }

    /// The same controller for another window.
    pub(crate) fn for_window(&self, window: WindowKey) -> Self {
        Self {
            shared: self.shared.clone(),
            window,
        }
    }

    /// The window this handle acts in.
    pub(crate) fn window(&self) -> WindowKey {
        self.window
    }

    /// Change controller bookkeeping the other windows do not draw (see
    /// [`Shared::bookkeep`]).
    pub(crate) fn bookkeep(&self) -> SharedMut<'_, BrowserController> {
        self.enter();
        self.shared.bookkeep()
    }

    /// Read the controller in this window's context.
    pub(crate) fn read(&self) -> SharedRef<'_, BrowserController> {
        self.enter();
        self.shared.read()
    }

    /// Change the controller in this window's context.
    pub(crate) fn get(&self) -> SharedMut<'_, BrowserController> {
        self.enter();
        self.shared.get()
    }

    /// Switch the controller to this window's context (bookkeeping, not a
    /// change other windows must see). Accesses within one statement all
    /// come from the same window, so only the first one switches.
    fn enter(&self) {
        let current = self.shared.read().current_window();
        if current != self.window {
            if let Err(error) = self.shared.borrow_raw().view_window(self.window) {
                log::warn!("controller access for a closed window: {error}");
            }
        }
    }
}
