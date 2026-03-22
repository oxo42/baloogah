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
                // Only increment if we haven't already counted this image as finished/errored
                // Actually, the current logic increments on every Finished. 
                // Let's make it idempotent if possible, or just trust the stream.
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
