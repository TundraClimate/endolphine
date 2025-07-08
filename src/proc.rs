pub mod view;

use crate::state::State;
use std::sync::Arc;

pub trait Runnable: Send + Sync {
    fn run(&'static self, state: Arc<State>, ctx: CommandContext);
}

pub struct CommandContext {
    prenum: Option<usize>,
}

impl CommandContext {
    pub fn new(prenum: Option<usize>) -> Self {
        Self { prenum }
    }
}

pub struct Command<F: Fn(Arc<State>, CommandContext)>(pub F);
pub struct Acommand<F: Fn(Arc<State>, CommandContext)>(pub F);

impl<F: Fn(Arc<State>, CommandContext) + Send + Sync> Runnable for Command<F> {
    fn run(&self, state: Arc<State>, ctx: CommandContext) {
        (self.0)(state, ctx)
    }
}

impl<F: Fn(Arc<State>, CommandContext) + Send + Sync> Runnable for Acommand<F> {
    fn run(&'static self, state: Arc<State>, ctx: CommandContext) {
        use tokio::task;

        task::spawn_blocking(|| (self.0)(state, ctx));
    }
}
