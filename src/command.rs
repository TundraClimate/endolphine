use crate::state::State;
use std::sync::Arc;

pub trait Runnable {
    fn run(&'static self, state: Arc<State>);
}

pub struct Command(pub fn(Arc<State>));
pub struct Acommand(pub fn(Arc<State>));

impl Runnable for Command {
    fn run(&self, state: Arc<State>) {
        (self.0)(state)
    }
}

impl Runnable for Acommand {
    fn run(&'static self, state: Arc<State>) {
        use tokio::task;

        task::spawn_blocking(|| (self.0)(state));
    }
}
