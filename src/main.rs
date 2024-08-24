use std::sync::Arc;

use crate::docker::docker_client;
use crate::docker::image_tags_for_running_images;

use docker::download_images;

mod docker;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(docker_client()?);

    let images = image_tags_for_running_images(Arc::clone(&client)).await?;
    // println!("{images:?}");
    // let images = {
    //     use std::collections::HashMap;
    //     let mut map = HashMap::new();
    //     map.insert(
    //         "containous/whoami".to_owned(),
    //         "containous_whoami".to_owned(),
    //     );
    //     map
    // };
    let images_strs: Vec<&str> = images //
        .keys()
        .map(|i| &**i)
        .collect();
    let downloaded_images = download_images(Arc::clone(&client), &images_strs).await;
    for i in downloaded_images {
        println!("{i}");
    }

    Ok(())
}
