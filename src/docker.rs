use anyhow::bail;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use docker_api::{
    models::{ImageBuildChunk, ProgressDetail},
    opts::PullOpts,
    Docker,
};
use futures::StreamExt;

pub async fn image_tags_for_running_images(
    client: Arc<Docker>,
) -> anyhow::Result<HashMap<String, String>> {
    let containers = client.containers().list(&Default::default()).await?;
    let tags: HashMap<String, String> = containers //
        .into_iter()
        .map(|c| (c.image.unwrap(), c.names.unwrap().first().unwrap().clone()))
        .collect();
    Ok(tags)
}

#[derive(Debug, Clone)]
pub enum Status {
    Update {
        image: String,
        status: String,
        #[allow(dead_code)]
        id: Option<String>,
        #[allow(dead_code)]
        progress: Option<String>,
        #[allow(dead_code)]
        progress_detail: Option<ProgressDetail>,
    },
    Error {
        image: String,
        error: String,
    },
    Finished {
        image: String,
        #[allow(dead_code)]
        did_download: bool,
    },
}

impl Status {
    fn from_chunk(image: String, chunk: ImageBuildChunk) -> anyhow::Result<Self> {
        let s = match chunk {
            ImageBuildChunk::Update { .. } => bail!("expected pullstatus"),
            ImageBuildChunk::Error { error, .. } => Self::Error {
                image,
                error,
            },
            ImageBuildChunk::Digest { .. } => bail!("expected pullstatus"),
            ImageBuildChunk::PullStatus {
                status,
                id,
                progress,
                progress_detail,
            } => Self::Update {
                image,
                status,
                id,
                progress,
                progress_detail,
            },
        };
        Ok(s)
    }

    fn finished(image: String, did_download: bool) -> Self {
        Self::Finished {
            image,
            did_download,
        }
    }
}

fn did_download(status: &str) -> bool {
    status.contains("Downloaded newer image")
}

pub async fn pull_image(client: Arc<Docker>, tx: mpsc::Sender<Status>, image: String) {
    let opts = PullOpts::builder().image(&image).build();
    let images = client.images();
    let mut stream = images.pull(&opts);
    let mut has_downloaded = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                let status = Status::from_chunk(image.clone(), chunk).expect("status to parse");
                if let Status::Update { status, .. } = &status {
                    has_downloaded = has_downloaded || did_download(status);
                }
                tx.send(status).await.expect("send to work");
            }
            Err(e) => {
                tx.send(Status::Error {
                    image: image.clone(),
                    error: e.to_string(),
                })
                .await
                .expect("send to work");
            }
        };
    }

    tx.send(Status::finished(image, has_downloaded))
        .await
        .expect("final send to work");
}

pub fn docker_client() -> docker_api::Result<Docker> {
    Docker::new("unix:///var/run/docker.sock")
}

pub fn download_images(
    client: Arc<Docker>,
    images: &[&str],
) -> (
    mpsc::Receiver<Status>,
    Vec<tokio::task::JoinHandle<()>>,
) {
    let mut join_handles = Vec::with_capacity(images.len());

    let (tx, rx) = mpsc::channel(100);
    for image in images {
        join_handles.push(tokio::spawn(pull_image(
            Arc::clone(&client),
            tx.clone(),
            image.to_string(),
        )));
    }

    (rx, join_handles)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pass() {
        println!("whoop");
    }
}
