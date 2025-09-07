use eframe::egui;
use jobtracker_core::{JobStatus, JobStore};
use strum::IntoEnumIterator;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    let mut job_app = JobApp::default();
    let _ = job_app.store.list_jobs().unwrap();
    eframe::run_native(
        "Job Tracker",
        options,
        Box::new(|_cc| Ok(Box::new(job_app))),
    )
}

#[derive(Default)]
struct JobApp {
    store: JobStore,
    new_company: String,
    new_role: String,
}

impl eframe::App for JobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Job Application Tracker");

            ui.separator();

            // Add new job form
            ui.horizontal(|ui| {
                ui.label("Company:");
                ui.text_edit_singleline(&mut self.new_company);

                ui.label("Role:");
                ui.text_edit_singleline(&mut self.new_role);

                if ui.button("Add").clicked()
                    && !self.new_company.is_empty()
                    && !self.new_role.is_empty()
                {
                    self.store
                        .add_job(self.new_company.clone(), self.new_role.clone())
                        .unwrap();
                    self.new_company.clear();
                    self.new_role.clear();
                }
            });

            ui.separator();

            // Track pending actions
            let mut to_remove: Option<usize> = None;
            let mut to_update: Option<(u32, JobStatus)> = None;

            // List jobs
            for (i, job) in self.store.jobs.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}  {}  {}  {}",
                            job.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            job.company,
                            job.role,
                            job.status,
                        ));

                        // Status dropdown
                        let mut selected_status = job.status.clone();
                        egui::ComboBox::from_label("Status")
                            .selected_text(selected_status.to_string())
                            .show_ui(ui, |ui| {
                                for status in JobStatus::iter() {
                                    if ui
                                        .selectable_value(
                                            &mut selected_status,
                                            status.clone(),
                                            status.to_string(),
                                        )
                                        .clicked()
                                    {
                                        // Queue update, but don't apply it yet
                                        to_update = Some((job.id, status));
                                    }
                                }
                            });

                        // Delete button
                        if ui.button("Delete").clicked() {
                            to_remove = Some(i);
                        }
                    });
                });
            }

            // Apply updates AFTER the loop to avoid borrow conflicts
            if let Some((id, new_status)) = to_update {
                self.store.update_status(id, new_status).unwrap();
            }
            if let Some(index) = to_remove {
                self.store.delete_job(index).unwrap();
            }
        });
    }
}
