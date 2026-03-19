use std::time::Instant;

use super::worker::FrameMessage;

/// Tracks per-frame latency and publishes 1-second peak values.
///
/// Accumulates per-frame max values across a 1-second window, then
/// snapshots them into `published_*` fields.  Every 10 snapshots the
/// 10-second report max is printed to stdout and reset.
pub struct LatencyTracker {
    // -- 1-second window accumulators --
    window_max_total_ms: f32,
    window_max_queue_ms: f32,
    window_max_uvc_wait_ms: f32,
    window_max_decode_ms: f32,
    window_max_rgb_pack_ms: f32,
    window_start: Instant,

    // -- Published (snapshot of last completed window) --
    pub total_ms: f32,
    pub queue_ms: f32,
    pub uvc_wait_ms: f32,
    pub decode_ms: f32,
    pub rgb_pack_ms: f32,

    // -- 10-second report accumulators --
    report_max_total_ms: f32,
    report_max_queue_ms: f32,
    report_max_uvc_wait_ms: f32,
    report_max_decode_ms: f32,
    report_max_rgb_pack_ms: f32,
    report_tick: u32,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self {
            window_max_total_ms: 0.0,
            window_max_queue_ms: 0.0,
            window_max_uvc_wait_ms: 0.0,
            window_max_decode_ms: 0.0,
            window_max_rgb_pack_ms: 0.0,
            window_start: Instant::now(),
            total_ms: 0.0,
            queue_ms: 0.0,
            uvc_wait_ms: 0.0,
            decode_ms: 0.0,
            rgb_pack_ms: 0.0,
            report_max_total_ms: 0.0,
            report_max_queue_ms: 0.0,
            report_max_uvc_wait_ms: 0.0,
            report_max_decode_ms: 0.0,
            report_max_rgb_pack_ms: 0.0,
            report_tick: 0,
        }
    }
}

impl LatencyTracker {
    /// Record a single frame's timing data.
    pub fn record(&mut self, frame: &FrameMessage) {
        let now = Instant::now();
        let total_delay_ms = now
            .saturating_duration_since(frame.uvc_ready_at)
            .as_secs_f32()
            * 1000.0;
        let queue_delay_ms = now
            .saturating_duration_since(frame.app_ready_at)
            .as_secs_f32()
            * 1000.0;

        self.window_max_total_ms = self.window_max_total_ms.max(total_delay_ms);
        self.window_max_queue_ms = self.window_max_queue_ms.max(queue_delay_ms);
        self.window_max_uvc_wait_ms = self.window_max_uvc_wait_ms.max(frame.uvc_wait_ms);
        self.window_max_decode_ms = self.window_max_decode_ms.max(frame.decode_ms);
        self.window_max_rgb_pack_ms = self.window_max_rgb_pack_ms.max(frame.rgb_pack_ms);

        if now
            .saturating_duration_since(self.window_start)
            .as_secs_f32()
            >= 1.0
        {
            self.publish_window();
            self.maybe_print_report();
            self.reset_window(now);
        }
    }

    fn publish_window(&mut self) {
        self.total_ms = self.window_max_total_ms;
        self.queue_ms = self.window_max_queue_ms;
        self.uvc_wait_ms = self.window_max_uvc_wait_ms;
        self.decode_ms = self.window_max_decode_ms;
        self.rgb_pack_ms = self.window_max_rgb_pack_ms;

        self.report_max_total_ms = self.report_max_total_ms.max(self.total_ms);
        self.report_max_queue_ms = self.report_max_queue_ms.max(self.queue_ms);
        self.report_max_uvc_wait_ms = self.report_max_uvc_wait_ms.max(self.uvc_wait_ms);
        self.report_max_decode_ms = self.report_max_decode_ms.max(self.decode_ms);
        self.report_max_rgb_pack_ms = self.report_max_rgb_pack_ms.max(self.rgb_pack_ms);
    }

    fn maybe_print_report(&mut self) {
        self.report_tick += 1;
        if self.report_tick >= 10 {
            println!(
                "[LATENCY] 10s max total={:.2}ms wait={:.2}ms decode={:.2}ms rgb={:.2}ms queue={:.2}ms",
                self.report_max_total_ms,
                self.report_max_uvc_wait_ms,
                self.report_max_decode_ms,
                self.report_max_rgb_pack_ms,
                self.report_max_queue_ms,
            );
            self.report_tick = 0;
            self.report_max_total_ms = 0.0;
            self.report_max_queue_ms = 0.0;
            self.report_max_uvc_wait_ms = 0.0;
            self.report_max_decode_ms = 0.0;
            self.report_max_rgb_pack_ms = 0.0;
        }
    }

    fn reset_window(&mut self, now: Instant) {
        self.window_max_total_ms = 0.0;
        self.window_max_queue_ms = 0.0;
        self.window_max_uvc_wait_ms = 0.0;
        self.window_max_decode_ms = 0.0;
        self.window_max_rgb_pack_ms = 0.0;
        self.window_start = now;
    }
}
