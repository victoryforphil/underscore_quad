use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::camera::CameraCapture;
use crate::fps::FpsCounter;

#[derive(Debug)]
pub struct FrameMessage {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
    pub fps: f32,
    pub uvc_wait_ms: f32,
    pub decode_ms: f32,
    pub rgb_pack_ms: f32,
    pub uvc_ready_at: Instant,
    pub app_ready_at: Instant,
}

pub struct CaptureWorker {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    rx: Receiver<FrameMessage>,
}

impl CaptureWorker {
    pub fn start(device: String, width: u32, height: u32, fps: u32) -> Result<Self> {
        let mut capture = CameraCapture::open(&device, width, height, fps)?;
        let (tx, rx) = sync_channel::<FrameMessage>(2);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);

        let join = thread::spawn(move || {
            capture_loop(
                &mut capture,
                width as usize,
                height as usize,
                tx,
                stop_thread,
            );
        });

        Ok(Self {
            stop,
            join: Some(join),
            rx,
        })
    }

    pub fn latest_frame(&self) -> Option<FrameMessage> {
        let mut latest = None;
        loop {
            match self.rx.try_recv() {
                Ok(frame) => latest = Some(frame),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => return latest,
            }
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for CaptureWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn capture_loop(
    capture: &mut CameraCapture,
    width: usize,
    height: usize,
    tx: SyncSender<FrameMessage>,
    stop: Arc<AtomicBool>,
) {
    let mut pixels_u32 = vec![0_u32; width * height];
    let mut rgb_bytes = vec![0_u8; width * height * 3];
    let mut fps_counter = FpsCounter::new(Duration::from_millis(500));

    while !stop.load(Ordering::Relaxed) {
        let timing = match capture.capture_to_u32_timed(&mut pixels_u32) {
            Ok(timing) => timing,
            Err(_) => continue,
        };

        let rgb_pack_start = Instant::now();
        convert_u32_to_rgb_bytes(&pixels_u32, &mut rgb_bytes);
        let rgb_pack_ms = rgb_pack_start.elapsed().as_secs_f32() * 1000.0;

        let msg = FrameMessage {
            width,
            height,
            rgb: rgb_bytes.clone(),
            fps: fps_counter.tick(),
            uvc_wait_ms: timing.uvc_wait.as_secs_f32() * 1000.0,
            decode_ms: timing.decode_and_convert.as_secs_f32() * 1000.0,
            rgb_pack_ms,
            uvc_ready_at: timing.frame_ready_at,
            app_ready_at: Instant::now(),
        };

        let _ = tx.try_send(msg);
    }
}

fn convert_u32_to_rgb_bytes(src: &[u32], dst: &mut [u8]) {
    for (idx, px) in src.iter().enumerate() {
        let base = idx * 3;
        dst[base] = ((px >> 16) & 0xff) as u8;
        dst[base + 1] = ((px >> 8) & 0xff) as u8;
        dst[base + 2] = (px & 0xff) as u8;
    }
}
