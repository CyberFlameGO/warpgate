use crate::helpers::fs::secure_file;

use super::{Error, Result};
use bytes::{Bytes, BytesMut};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, Mutex};
use tracing::*;
use warpgate_db_entities::Recording;

#[derive(Clone)]
pub struct RecordingWriter {
    sender: mpsc::Sender<Bytes>,
}

impl RecordingWriter {
    pub(crate) async fn new(
        path: PathBuf,
        model: Recording::Model,
        db: Arc<Mutex<DatabaseConnection>>,
    ) -> Result<Self> {
        let file = File::create(&path).await?;
        secure_file(&path)?;
        let mut writer = BufWriter::new(file);
        let (sender, mut receiver) = mpsc::channel::<Bytes>(1024);
        tokio::spawn(async move {
            if let Err(error) = async {
                let mut last_flush = Instant::now();
                loop {
                    if Instant::now() - last_flush > Duration::from_secs(5) {
                        last_flush = Instant::now();
                        writer.flush().await?;
                    }
                    tokio::select! {
                        data = receiver.recv() => match data {
                            Some(bytes) => {
                                writer.write_all(&bytes).await?;
                            }
                            None => break,
                        },
                        _ = tokio::time::sleep(Duration::from_millis(5000)) => ()
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                error!(%error, ?path, "Failed to write recording");
            }

            if let Err(error) = async {
                writer.flush().await?;

                use sea_orm::ActiveValue::Set;
                let id = model.id;
                let db = db.lock().await;
                let recording = Recording::Entity::find_by_id(id)
                    .one(&*db)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Recording not found"))?;
                let mut model: Recording::ActiveModel = recording.into();
                model.ended = Set(Some(chrono::Utc::now()));
                model.update(&*db).await?;
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                error!(%error, ?path, "Failed to write recording");
            }
        });

        Ok(RecordingWriter { sender })
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.sender
            .send(BytesMut::from(data).freeze())
            .await
            .map_err(|_| Error::Closed)?;
        Ok(())
    }
}
