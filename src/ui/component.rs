use std::ops::ControlFlow;
use std::pin::Pin;
use crossterm::event::Event;
use crate::app::AppState;
use tokio::sync::mpsc;
use crate::app::worker::commands::WorkerCmd;

pub trait Component {
    fn handle_event<'a>(&'a mut self, state: &'a mut AppState, cmd_tx: &'a mpsc::Sender<WorkerCmd>, ev: Event) -> Pin<Box<dyn Future<Output = ControlFlow<()>>  + '_>>;
}