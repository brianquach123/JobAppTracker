use chrono::Utc;
use eframe::egui::ViewportBuilder;
use jobtracker_core::{JobApp, APP_NAME, WINDOW_HEIGHT, WINDOW_WIDTH};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(true),
        ..Default::default()
    };

    let mut job_app = JobApp {
        last_refresh: Utc::now(),
        ..Default::default()
    };

    job_app.store.load_from_file().unwrap();
    eframe::run_native(APP_NAME, options, Box::new(|_cc| Ok(Box::new(job_app))))
}
