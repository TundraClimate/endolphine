use crossterm::event;
use crossterm::event::Event;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub enum Signal {
    Shatdown,
    Pause,
}

pub fn spawn() -> (Receiver<Event>, Sender<Signal>) {
    let (tx, rx) = mpsc::channel::<Event>(100);
    let (sender, mut receiver) = mpsc::channel::<Signal>(100);
    let mut paused = false;

    tokio::spawn(async move {
        loop {
            if let Ok(signal) = receiver.try_recv() {
                match signal {
                    Signal::Pause => paused = !paused,
                    Signal::Shatdown => break,
                }
            }

            if paused {
                continue;
            }

            if let Ok(event) = tokio::task::spawn_blocking(|| event::read()).await {
                if let Ok(event) = event {
                    tx.send(event).await.expect("buffer capacity reached.");
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    (rx, sender)
}
