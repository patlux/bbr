//! Public AppKit behind-window material. This is a sibling *behind* the
//! GPUI rendering view, never a WebKit background or an input surface.
//! Ownership belongs to one browser shell, including windows with no tabs.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowOrderingMode,
};
use objc2_foundation::NSPoint;

define_class!(
    #[unsafe(super = NSVisualEffectView)]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRChromeMaterial"]
    struct MaterialView;

    impl MaterialView {
        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
            None
        }
    }
);

/// One window's material. It retains only its effect view, not the window.
pub struct ChromeMaterial {
    view: Retained<MaterialView>,
    mask: super::chrome_mask::CardMask,
    dark: Option<bool>,
}

impl ChromeMaterial {
    /// Install behind the live GPUI root without replacing its content view,
    /// layer, responder chain, children or constraints.
    ///
    /// # Safety
    /// `ns_view` must be this window's live GPUI NSView on the main thread.
    pub unsafe fn install(mtm: MainThreadMarker, ns_view: *mut std::ffi::c_void) -> Option<Self> {
        // SAFETY: caller guarantees a live main-thread NSView.
        let root = unsafe { ns_view.cast::<NSView>().as_ref() }?;
        let parent = unsafe { root.superview() }?;
        let allocated = MaterialView::alloc(mtm).set_ivars(());
        // SAFETY: public NSView designated initializer on our allocated subclass.
        let view: Retained<MaterialView> =
            unsafe { msg_send![super(allocated), initWithFrame: root.frame()] };
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        view.setMaterial(NSVisualEffectMaterial::Sidebar);
        view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        view.setState(NSVisualEffectState::FollowsWindowActiveState);
        parent.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Below, Some(root));
        let Some(mask) = super::chrome_mask::CardMask::new(root, &view) else {
            view.removeFromSuperview();
            return None;
        };
        Some(Self {
            view,
            mask,
            dark: None,
        })
    }

    /// Match explicit BBR appearance choices as well as system changes. This
    /// affects only the material, never the window or WKWebView appearance.
    pub fn update(&mut self, dark: bool, card: [f32; 4], radius: f32, tint: [f32; 4]) {
        if self.dark != Some(dark) {
            // SAFETY: public immutable appearance-name constants.
            let name = unsafe {
                if dark {
                    NSAppearanceNameDarkAqua
                } else {
                    NSAppearanceNameAqua
                }
            };
            let appearance = NSAppearance::appearanceNamed(name);
            self.view.setAppearance(appearance.as_deref());
            self.dark = Some(dark);
        }
        self.mask.update(card, radius, tint);
    }

    /// Solid chrome covers this layer in accessibility fallback mode. Hide
    /// the effect as well, avoiding unnecessary backdrop work in that mode.
    pub fn set_enabled(&self, enabled: bool) {
        if self.view.isHidden() == enabled {
            self.view.setHidden(!enabled);
            self.mask.set_enabled(enabled);
        }
    }

    /// Read the native state, not merely the requested policy.
    #[cfg(feature = "testkit")]
    pub fn observation(&self) -> MaterialObservation {
        let frame = self.view.frame();
        // SAFETY: the owned view is accessed only on its AppKit main thread.
        let parent = unsafe { self.view.superview() };
        let window = self.view.window();
        MaterialObservation {
            attached: parent.is_some() && window.is_some(),
            enabled: !self.view.isHidden(),
            card_mask_valid: self.mask.clips_only_card_corners(),
            behind_window: self.view.blendingMode() == NSVisualEffectBlendingMode::BehindWindow,
            follows_activation: self.view.state() == NSVisualEffectState::FollowsWindowActiveState,
            window_opaque: window.is_none_or(|window| window.isOpaque()),
            size: [frame.size.width as f32, frame.size.height as f32],
        }
    }
}

impl Drop for ChromeMaterial {
    fn drop(&mut self) {
        self.view.removeFromSuperview();
    }
}

#[cfg(feature = "testkit")]
pub struct MaterialObservation {
    pub attached: bool,
    pub enabled: bool,
    pub card_mask_valid: bool,
    pub behind_window: bool,
    pub follows_activation: bool,
    pub window_opaque: bool,
    pub size: [f32; 2],
}
