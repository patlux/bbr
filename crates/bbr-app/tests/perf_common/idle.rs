//! v2 motion idle scenario (task packet `v2-motion-core`, DESIGN §18.5):
//! after the warm interaction matrix settles, the optimized app must request
//! no animation frames while nothing transient is presented. Only animated
//! moments request frames, so the budget is **zero** animation elements
//! built across a run of forced frames — a render-side count that holds
//! whether or not the display link ticks.

use bbr_testkit::protocol::Request;
use bbr_testkit::Client;

/// Forced frames sampled for the idle budget.
const IDLE_FRAMES: usize = 5;

/// Assert the idle budget: no presented moment and zero animation elements
/// built across [`IDLE_FRAMES`] forced frames, each of which must render.
pub(crate) fn assert_idle_requests_no_frames(client: &mut Client) {
    // Let any in-flight moment (a finishing load, a last pulse) end first:
    // the scenario measures the settled shell, bounded by the deadline.
    let settle = super::benchmarks::deadline();
    let start = loop {
        let motion = client.snapshot().expect("idle start").motion;
        if motion.moments.is_empty() {
            break motion;
        }
        assert!(
            std::time::Instant::now() < settle,
            "the shell never settled: {:?}",
            motion.moments
        );
        std::thread::yield_now();
    };
    let mut renders = start.root_renders;
    for frame in 0..IDLE_FRAMES {
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush idle frame");
        let motion = client.snapshot().expect("idle frame").motion;
        assert!(motion.root_renders > renders, "idle frame {frame} rendered");
        renders = motion.root_renders;
        assert_eq!(
            motion.animated_elements, start.animated_elements,
            "idle frame {frame} built an animation element (requests frames)"
        );
    }
}
