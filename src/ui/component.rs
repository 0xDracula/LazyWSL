use crate::app::AppState;
use crate::app::worker::commands::WorkerCmd;
use crossterm::event::Event;
use std::ops::ControlFlow;
use std::pin::Pin;
use tokio::sync::mpsc;

pub trait Component {
    fn handle_event<'a>(
        &'a mut self,
        state: &'a mut AppState,
        cmd_tx: &'a mpsc::Sender<WorkerCmd>,
        ev: Event,
    ) -> Pin<Box<dyn Future<Output = ControlFlow<()>> + '_>>;
}
