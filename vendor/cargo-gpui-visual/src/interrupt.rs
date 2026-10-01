//! CLI-only signal ownership. Library callers supply their own cancellation flag.
use anyhow::Result;
use std::sync::{atomic::AtomicBool, Arc};

pub(crate) struct Interrupts {
    pub cancelled: Arc<AtomicBool>,
    #[cfg(unix)]
    registrations: Vec<signal_hook::SigId>,
}
impl Interrupts {
    pub fn install() -> Result<Self> {
        let mut result = Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            #[cfg(unix)]
            registrations: Vec::new(),
        };
        #[cfg(unix)]
        for signal in [
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGHUP,
        ] {
            result.registrations.push(signal_hook::flag::register(
                signal,
                result.cancelled.clone(),
            )?);
        }
        Ok(result)
    }
}
impl Drop for Interrupts {
    fn drop(&mut self) {
        #[cfg(unix)]
        for id in self.registrations.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}
