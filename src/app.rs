use std::collections::HashMap;
use crate::docker::Status;

pub struct App {
    pub total_images: usize,
    pub completed_images: usize,
    pub image_order: Vec<String>,
    pub image_progress: HashMap<String, f32>,
    pub image_errors: HashMap<String, String>,
    pub errors: Vec<String>,
    pub tick: u64,
}

impl App {
    pub fn new(images: &[&str]) -> Self {
        let mut image_order = Vec::new();
        let mut image_progress = HashMap::new();
        let mut image_errors = HashMap::new();
        for image in images {
            let img_str = image.to_string();
            image_order.push(img_str.clone());
            image_progress.insert(img_str.clone(), 0.0);
            image_errors.insert(img_str, String::new());
        }

        Self {
            total_images: images.len(),
            completed_images: 0,
            image_order,
            image_progress,
            image_errors,
            errors: Vec::new(),
            tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn handle_status(&mut self, status: Status) {
        match status {
            Status::Finished { image, .. } => {
                self.completed_images += 1;
                if let Some(p) = self.image_progress.get_mut(&image) {
                    *p = 1.0;
                }
            }
            Status::Update {
                image,
                status,
                progress_detail,
                ..
            } => {
                if let Some(detail) = progress_detail {
                    if let (Some(current), Some(total)) = (detail.current, detail.total) {
                        if total > 0 {
                            let progress = current as f32 / total as f32;
                            if let Some(p) = self.image_progress.get_mut(&image) {
                                *p = progress;
                            }
                        }
                    }
                } else if status.contains("Downloaded newer image")
                    || status.contains("Image is up to date")
                {
                    if let Some(p) = self.image_progress.get_mut(&image) {
                        *p = 1.0;
                    }
                }
            }
            Status::Error { image, error } => {
                self.errors.push(format!("{}: {}", image, error));
                if let Some(e) = self.image_errors.get_mut(&image) {
                    *e = error;
                }
                self.completed_images += 1;
            }
        }
    }
}

pub fn shorten_name(name: &str) -> &str {
    name.split('/').last().unwrap_or(name)
}

pub fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        return name.to_string();
    }
    if max_len < 3 {
        return "...".to_string();
    }
    let side_len = (max_len.saturating_sub(3)) / 2;
    let start = &name[..side_len];
    let end = &name[name.len() - side_len..];
    format!("{}...{}", start, end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use docker_api::models::ProgressDetail;

    #[test]
    fn test_shorten_name() {
        assert_eq!(shorten_name("ghcr.io/home-assistant/home-assistant:stable"), "home-assistant:stable");
        assert_eq!(shorten_name("nginx:latest"), "nginx:latest");
        assert_eq!(shorten_name("myrepo/myimage"), "myimage");
    }

    #[test]
    fn test_truncate_name() {
        let name = "abcdefghij";
        assert_eq!(truncate_name(name, 10), "abcdefghij");
        assert_eq!(truncate_name(name, 7), "ab...ij");
        assert_eq!(truncate_name(name, 5), "a...j");
        assert_eq!(truncate_name(name, 3), "...");
    }

    #[test]
    fn test_app_initialization() {
        let images = vec!["image1", "image2"];
        let app = App::new(&images);
        assert_eq!(app.total_images, 2);
        assert_eq!(app.completed_images, 0);
        assert_eq!(app.image_order, vec!["image1".to_string(), "image2".to_string()]);
        assert_eq!(app.tick, 0);
    }

    #[test]
    fn test_app_handle_status_update() {
        let mut app = App::new(&["image1"]);
        let detail = ProgressDetail {
            current: Some(50),
            total: Some(100),
        };
        app.handle_status(Status::Update {
            image: "image1".to_string(),
            status: "Downloading".to_string(),
            id: None,
            progress: None,
            progress_detail: Some(detail),
        });
        assert_eq!(app.image_progress.get("image1"), Some(&0.5));
    }

    #[test]
    fn test_app_handle_status_finished() {
        let mut app = App::new(&["image1"]);
        app.handle_status(Status::Finished {
            image: "image1".to_string(),
            did_download: true,
        });
        assert_eq!(app.completed_images, 1);
        assert_eq!(app.image_progress.get("image1"), Some(&1.0));
    }

    #[test]
    fn test_app_handle_status_error() {
        let mut app = App::new(&["image1"]);
        app.handle_status(Status::Error {
            image: "image1".to_string(),
            error: "Failed to pull".to_string(),
        });
        assert_eq!(app.completed_images, 1);
        assert_eq!(app.errors.len(), 1);
        assert_eq!(app.image_errors.get("image1").unwrap(), "Failed to pull");
    }
}
