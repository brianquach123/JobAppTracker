use eframe::egui::{self, TextEdit};
use egui_plot::PlotPoint;
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
    search_text: String,
}

impl eframe::App for JobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Job Application Tracker");
            ui.separator();

            ui.horizontal(|ui| {
                // Search box
                ui.label("Search by company:");
                ui.add(TextEdit::singleline(&mut self.search_text));

                // Refresh button
                if ui.add(egui::Button::new("Refresh")).clicked() {
                    println!("PRessed");
                    let _ = self.store.list_jobs();
                }
            });

            ui.separator();

            // ----------------------------
            // Add new job form
            // ----------------------------
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

            // ----------------------------
            // Weekly bar chart (last 7 days)
            // ----------------------------
            {
                use chrono::{Duration, Local, NaiveDate};
                use eframe::egui::Color32;
                use egui_plot::{Bar, BarChart, Plot, Text};
                use std::collections::HashMap;

                let today = Local::now().date_naive();
                let last_7_days: Vec<NaiveDate> = (0..7)
                    .rev() // oldest day first
                    .map(|i| today - Duration::days(i))
                    .collect();

                // Count jobs per day
                let mut counts: HashMap<NaiveDate, usize> = HashMap::new();
                for job in &self.store.jobs {
                    let job_date = job.timestamp.with_timezone(&Local).date_naive();
                    if last_7_days.contains(&job_date) {
                        *counts.entry(job_date).or_default() += 1;
                    }
                }

                // Prepare values for plotting
                let values: Vec<(f64, f64)> = last_7_days
                    .iter()
                    .enumerate()
                    .map(|(i, date)| (i as f64, *counts.get(date).unwrap_or(&0) as f64))
                    .collect();

                let bars: Vec<Bar> = values
                    .iter()
                    .map(|&(x, y)| Bar::new(x, y).fill(Color32::from_rgb(100, 150, 250)))
                    .collect();

                let chart = BarChart::new(bars).width(0.6);

                Plot::new("weekly_jobs").height(150.0).show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);

                    // Optional: add x-axis labels (MM-DD)
                    for (i, date) in last_7_days.iter().enumerate() {
                        let label = date.format("%m-%d").to_string();
                        plot_ui.text(Text::new(PlotPoint::new(i as f64, -0.5), label));
                    }
                });
            }

            ui.separator();

            // ----------------------------
            // Scrollable job list grid
            // ----------------------------
            let mut to_remove: Option<usize> = None;
            let mut to_update: Option<(u32, JobStatus)> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("jobs_grid").striped(true).show(ui, |ui| {
                        // Header row
                        ui.label("ID");
                        ui.label("Timestamp");
                        ui.label("Company");
                        ui.label("Role");
                        ui.label("Status");
                        ui.end_row();

                        // Filter jobs
                        for (i, job) in self
                            .store
                            .jobs
                            .iter_mut()
                            .filter(|job| {
                                self.search_text.is_empty()
                                    || job
                                        .company
                                        .to_lowercase()
                                        .contains(&self.search_text.to_lowercase())
                            })
                            .enumerate()
                        {
                            ui.label(job.id.to_string());
                            ui.label(job.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                            ui.label(&job.company);
                            ui.label(&job.role);

                            let mut selected_status = job.status.clone();
                            egui::ComboBox::from_id_source(i)
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
                                            to_update = Some((job.id, status));
                                        }
                                    }
                                });

                            if ui.button("Delete").clicked() {
                                to_remove = Some(i);
                            }

                            ui.end_row();
                        }
                    });
                });

            // Apply pending updates AFTER the loop
            if let Some((id, new_status)) = to_update {
                self.store.update_status(id, new_status).unwrap();
            }
            if let Some(index) = to_remove {
                self.store.delete_job(index).unwrap();
            }
        });
    }
}
