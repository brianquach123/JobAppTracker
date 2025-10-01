use crate::{Job, JobStatus, CYAN, GRAY, GREEN, NAVY_BLUE, RED};
use eframe::egui::Color32;

impl Job {
    pub fn get_status_color_mapping(&self) -> Color32 {
        match self.status {
            JobStatus::Applied => NAVY_BLUE,
            JobStatus::Interview => CYAN,
            JobStatus::Offer => GREEN,
            JobStatus::Rejected => RED,
            JobStatus::Ghosted => GRAY,
        }
    }
}
