use eframe::egui;
use jobtracker_core::JobStore;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Job Tracker",
        options,
        Box::new(|_cc| Ok(Box::new(JobApp::default()))),
    )
}

struct JobApp {
    store: JobStore,
    new_company: String,
    new_role: String,
    status_options: Vec<&'static str>,
}

impl Default for JobApp {
    fn default() -> Self {
        Self {
            store: JobStore::default(),
            new_company: String::new(),
            new_role: String::new(),
            status_options: vec!["Applied", "Interview", "Offer", "Rejected"],
        }
    }
}

impl eframe::App for JobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Job Applications");

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
                        .add_job(self.new_company.clone(), self.new_role.clone());
                    self.new_company.clear();
                    self.new_role.clear();
                }
            });

            ui.separator();

            // Track which job to delete after iteration
            let mut to_remove: Option<usize> = None;

            // List jobs
            for (i, job) in self.store.jobs.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{} - {}", job.company, job.role));

                    // Status dropdown modifies job directly
                    egui::ComboBox::from_label("Status")
                        .selected_text(&job.status)
                        .show_ui(ui, |ui| {
                            for status in &self.status_options {
                                if ui
                                    .selectable_value(&mut job.status, status.to_string(), *status)
                                    .clicked()
                                {
                                    // job.status updated directly, no extra borrow
                                }
                            }
                        });

                    // Mark for deletion
                    if ui.button("Delete").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            // Remove job after iteration to avoid double mutable borrow
            if let Some(index) = to_remove {
                self.store.delete_job(index);
            }
        });
    }
}
