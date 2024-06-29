use crossterm::event;
use crossterm::event::Event;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub fn spawn() -> (Receiver<Event>, Sender<Option<()>>) {
    let (tx, rx) = mpsc::channel::<Event>(100);
    let (sender, mut receiver) = mpsc::channel::<Option<()>>(100);

    tokio::spawn(async move {
        loop {
            if let Ok(event) = event::read() {
                tx.send(event).await.expect("buffer capacity reached.");
                if let Some(res) = receiver.recv().await {
                    match res {
                        Some(_) => break,
                        None => {}
                    }
                }
            }
        }
    });

    (rx, sender)
}
