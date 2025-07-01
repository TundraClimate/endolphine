use crate::state::State;
use std::sync::Arc;

pub trait Runnable: Send + Sync {
    fn run(&'static self, state: Arc<State>, ctx: CommandContext);
}

pub struct CommandContext {
    prenum: Option<usize>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self { prenum: None }
    }
}

impl CommandContext {
    pub fn new(prenum: Option<usize>) -> Self {
        Self { prenum }
    }
}

pub struct Command(pub fn(Arc<State>, CommandContext));
pub struct Acommand(pub fn(Arc<State>, CommandContext));

impl Runnable for Command {
    fn run(&self, state: Arc<State>, ctx: CommandContext) {
        (self.0)(state, ctx)
    }
}

impl Runnable for Acommand {
    fn run(&'static self, state: Arc<State>, ctx: CommandContext) {
        use tokio::task;

        task::spawn_blocking(|| (self.0)(state, ctx));
    }
}
