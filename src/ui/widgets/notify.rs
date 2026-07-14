use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ratatui::style::Color;

use crate::ui::theme;

#[derive(Debug, Clone)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn color(&self) -> Color {
        match self {
            Level::Info => theme::ACCENT,
            Level::Warn => theme::INSTALLING,
            Level::Error => theme::ERROR,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Level::Info => "\u{f05a}",
            Level::Warn => "\u{f071}",
            Level::Error => "\u{f057}",
        }
    }
}

pub enum Anchor {
    TopRight,
    TopCenter,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub msg: String,
    pub level: Level,
    dismiss: Duration,
    started_at: Option<Instant>,
}

#[derive(Debug, Default)]
pub struct Toasts {
    queue: VecDeque<Toast>,
}

impl Toasts {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, msg: String, level: Level, dismiss_secs: u64) {
        self.queue.push_back(Toast {
            msg,
            level,
            dismiss: Duration::from_secs(dismiss_secs.max(1)),
            started_at: None,
        });
    }

    pub fn tick(&mut self) {
        if let Some(front) = self.queue.front_mut() {
            let started_at = *front.started_at.get_or_insert_with(Instant::now);
            if started_at + front.dismiss <= Instant::now() {
                self.queue.pop_front();
            }
        }
    }

    pub fn latest(&self) -> Option<&Toast> {
        self.queue.front()
    }
}
