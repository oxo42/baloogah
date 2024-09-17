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
pub struct Status {
    pub image: String,
    pub status: String,
    #[allow(dead_code)]
    pub id: Option<String>,
    #[allow(dead_code)]
    pub progress: Option<String>,
    #[allow(dead_code)]
    pub progress_detail: Option<ProgressDetail>,
    pub did_download: bool,
}

impl Status {
    fn from_chunk(image: String, chunk: ImageBuildChunk) -> anyhow::Result<Self> {
        let s = match chunk {
            ImageBuildChunk::Update { .. } => bail!("expected pullstatus"),
            ImageBuildChunk::Error { .. } => bail!("expected pullstatus"),
            ImageBuildChunk::Digest { .. } => bail!("expected pullstatus"),
            ImageBuildChunk::PullStatus {
                status,
                id,
                progress,
                progress_detail,
            } => Self {
                image,
                status,
                id,
                progress,
                progress_detail,
                did_download: false,
            },
        };
        Ok(s)
    }

    fn finished(image: String, did_download: bool) -> Self {
        Self {
            image,
            status: "Finished".to_owned(),
            id: None,
            progress: None,
            progress_detail: None,
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
                has_downloaded = has_downloaded || did_download(&status.status);
                tx.send(status).await.expect("send to work");
            }
            Err(e) => eprintln!("{e}"),
        };
    }

    println!("finished");
    tx.send(Status::finished(image, has_downloaded))
        .await
        .expect("final send to work");
}

pub fn docker_client() -> docker_api::Result<Docker> {
    Docker::new("unix:///var/run/docker.sock")
}

pub async fn download_images(client: Arc<Docker>, images: &[&str]) -> Vec<String> {
    let mut join_handles = Vec::with_capacity(images.len());
    let mut downloaded_images = Vec::with_capacity(images.len());

    let mut rx = {
        let (tx, rx) = mpsc::channel(32);
        for image in images {
            join_handles.push(tokio::spawn(pull_image(
                Arc::clone(&client),
                tx.clone(),
                image.to_string(),
            )));
        }
        rx
    };

    while let Some(status) = rx.recv().await {
        println!("got = {status:?}");
        if status.did_download {
            downloaded_images.push(status.image);
        }
    }

    for join_handle in join_handles {
        join_handle.await.unwrap();
    }

    downloaded_images.sort();
    downloaded_images
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pass() {
        println!("whoop");
    }
}
