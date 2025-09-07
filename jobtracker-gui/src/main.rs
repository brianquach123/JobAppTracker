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

const STATUSES: [&str; 4] = ["Applied", "Interview", "Offer", "Rejected"];

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
                // Push a unique ID scope for this job
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
                        egui::ComboBox::from_label("Status")
                            .selected_text(&job.status)
                            .show_ui(ui, |ui| {
                                for status in &STATUSES {
                                    if ui
                                        .selectable_value(
                                            &mut job.status,
                                            status.to_string(),
                                            *status,
                                        )
                                        .clicked()
                                    {
                                        // update directly
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

            // Remove job after iteration to avoid double mutable borrow
            if let Some(index) = to_remove {
                self.store.delete_job(index);
            }
        });
    }
}
