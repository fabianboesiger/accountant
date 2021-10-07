pub use shared::*;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time;
use tokio::{
    net::TcpStream,
    sync::mpsc::{error::SendError, UnboundedSender},
};

pub struct Accountant {
    sender: UnboundedSender<Message>,
}

impl Accountant {
    pub async fn connect() -> Self {
        log::info!("Starting accountant!");

        let (sender, mut receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                log::debug!("Starting sender.");
                match async {
                    let mut stream = TcpStream::connect("127.0.0.1:5000").await?;
                    log::debug!("Connected to server.");
                    while let Some(message) = receiver.recv().await {
                        let serialized = bincode::serialize(&message)?;
                        match async {
                            stream.write_all(&serialized).await?;
                            Ok::<(), Box<dyn std::error::Error>>(())
                        }
                        .await
                        {
                            Ok(()) => {}
                            result @ Err(_) => {
                                log::debug!("Got error while sending, requeueing message.");
                                sender_clone.send(message)?;
                                result?;
                            }
                        }
                    }

                    Ok::<(), Box<dyn std::error::Error>>(())
                }
                .await
                {
                    Ok(()) => log::error!("Sender terminated."),
                    Err(err) => log::error!("Sender got error {:?}", err),
                }
                interval.tick().await;
            }
        });

        Self { sender }
    }

    pub fn notify<M: Into<Message>>(&self, message: M) -> Result<(), SendError<Message>> {
        let message = message.into();
        log::debug!("Sending message {:?}", message);
        self.sender.send(message)?;
        Ok(())
    }
}
