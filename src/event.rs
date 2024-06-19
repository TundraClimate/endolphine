use crossterm::event;
use crossterm::event::Event;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;

pub fn spawn() -> (Receiver<Event>, Sender<()>) {
    let (tx, rx) = mpsc::channel::<Event>(100);
    let (shatdown, mut sd_signal) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            if let Ok(event) = tokio::task::spawn_blocking(|| event::read()).await {
                if let Ok(event) = event {
                    tx.send(event).await.expect("buffer capacity reached.");
                }
            }

            if let Ok(_) = sd_signal.try_recv() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    (rx, shatdown)
}
