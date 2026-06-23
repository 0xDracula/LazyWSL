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
    pub expires_at: Instant,
}

#[derive(Debug, Default)]
pub struct Toasts {
    current: Option<Toast>,
}

impl Toasts {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn push(&mut self, msg: String, level: Level, dismiss_secs: u64) {
        self.current = Some(Toast {
            msg,
            level,
            expires_at: Instant::now() + Duration::from_secs(dismiss_secs.max(1)),
        });
    }

    pub fn tick(&mut self) {
        if let Some(t) = &self.current
            && t.expires_at <= Instant::now()
        {
            self.current = None;
        }
    }
    pub fn latest(&self) -> Option<&Toast> {
        self.current.as_ref()
    }
}
