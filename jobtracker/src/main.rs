use chrono::Utc;
use jobtracker_core::{JobApp, APP_NAME};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    let mut job_app = JobApp {
        last_refresh: Utc::now(),
        ..Default::default()
    };
    job_app.store.load_from_file().unwrap();
    eframe::run_native(APP_NAME, options, Box::new(|_cc| Ok(Box::new(job_app))))
}
