use crate::{canvas, error::*, event_handler};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{self, Duration, Instant};

pub async fn process(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    loop {
        if event_handler::handle_event().await? {
            quit_flag.swap(true, Ordering::Relaxed);
            break;
        }
    }

    Ok(())
}

pub async fn ui(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    while !quit_flag.load(Ordering::Relaxed) {
        let start = Instant::now();

        {
            canvas::render()?;
        }

        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(50) {
            time::sleep(Duration::from_millis(50) - elapsed).await;
        }
    }

    Ok(())
}
