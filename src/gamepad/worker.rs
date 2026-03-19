use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::Result;

use crate::gamepad::{
    GamepadCollector, GamepadCollectorOptions, GamepadControlFrame, GamepadControlMapping,
    GamepadEvent, GamepadSnapshot,
};

#[derive(Debug, Clone)]
pub struct GamepadWorkerOptions {
    pub poll_timeout: Duration,
    pub collector: GamepadCollectorOptions,
    pub control_mapping: Option<GamepadControlMapping>,
}

impl Default for GamepadWorkerOptions {
    fn default() -> Self {
        Self {
            poll_timeout: Duration::from_millis(20),
            collector: GamepadCollectorOptions::default(),
            control_mapping: Some(GamepadControlMapping::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GamepadWorkerUpdate {
    pub events: Vec<GamepadEvent>,
    pub active_snapshot: Option<GamepadSnapshot>,
    pub control_frame: Option<GamepadControlFrame>,
}

pub struct GamepadWorker {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    rx: Receiver<GamepadWorkerUpdate>,
}

impl GamepadWorker {
    pub fn start(options: GamepadWorkerOptions) -> Result<Self> {
        let mut collector = GamepadCollector::with_options(options.collector)?;
        let initial_update = build_update(&collector, Vec::new(), options.control_mapping);
        let (tx, rx) = sync_channel::<GamepadWorkerUpdate>(4);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);

        let join = thread::spawn(move || {
            if let Some(update) = initial_update {
                let _ = tx.try_send(update);
            }
            worker_loop(&mut collector, options, tx, stop_thread);
        });

        Ok(Self {
            stop,
            join: Some(join),
            rx,
        })
    }

    pub fn latest_update(&self) -> Option<GamepadWorkerUpdate> {
        let mut latest = None;
        loop {
            match self.rx.try_recv() {
                Ok(update) => latest = Some(update),
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

impl Drop for GamepadWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn worker_loop(
    collector: &mut GamepadCollector,
    options: GamepadWorkerOptions,
    tx: SyncSender<GamepadWorkerUpdate>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        let events = collector.poll_blocking(Some(options.poll_timeout));
        if let Some(update) = build_update(collector, events, options.control_mapping) {
            let _ = tx.try_send(update);
        }
    }
}

fn build_update(
    collector: &GamepadCollector,
    events: Vec<GamepadEvent>,
    control_mapping: Option<GamepadControlMapping>,
) -> Option<GamepadWorkerUpdate> {
    let active_snapshot = collector.active_snapshot().cloned();
    if events.is_empty() && active_snapshot.is_none() {
        return None;
    }

    let control_frame = active_snapshot
        .as_ref()
        .and_then(|snapshot| control_mapping.map(|mapping| mapping.map_snapshot(snapshot)));

    Some(GamepadWorkerUpdate {
        events,
        active_snapshot,
        control_frame,
    })
}
