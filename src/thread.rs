use crate::error::*;
use std::sync::{atomic::AtomicBool, Arc};

pub async fn process(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    Ok(())
}

pub async fn ui(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    Ok(())
}
