use crossterm::event;
use crossterm::event::Event;
use std::error::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub struct EventThread {
    rc: Receiver<Event>,
    sd: Sender<Option<()>>,
}

impl EventThread {
    pub async fn shatdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.sd.send(Some(())).await?;
        Ok(())
    }

    pub async fn respond(&mut self) -> Result<(), Box<dyn Error>> {
        self.sd.send(None).await?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Event, mpsc::error::TryRecvError> {
        self.rc.try_recv()
    }
}

pub fn spawn() -> EventThread {
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
    EventThread { rc: rx, sd: sender }
}
