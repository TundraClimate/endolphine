use crate::error::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{self, Duration, Instant};

pub async fn process(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    loop {}
    Ok(())
}

pub async fn ui(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    while !quit_flag.load(Ordering::Relaxed) {
        let start = Instant::now();

        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(50) {
            time::sleep(Duration::from_millis(50) - elapsed).await;
        }
    }

    Ok(())
}
