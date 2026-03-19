use std::time::{Duration, Instant};

pub struct FpsCounter {
    period_start: Instant,
    period_frames: u32,
    update_every: Duration,
    fps: f32,
}

impl FpsCounter {
    pub fn new(update_every: Duration) -> Self {
        Self {
            period_start: Instant::now(),
            period_frames: 0,
            update_every,
            fps: 0.0,
        }
    }

    pub fn tick(&mut self) -> f32 {
        self.period_frames += 1;
        let elapsed = self.period_start.elapsed();

        if elapsed >= self.update_every {
            self.fps = self.period_frames as f32 / elapsed.as_secs_f32();
            self.period_frames = 0;
            self.period_start = Instant::now();
        }

        self.fps
    }
}
