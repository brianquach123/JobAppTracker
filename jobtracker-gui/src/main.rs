use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono::{Local, NaiveDateTime, TimeZone};
use eframe::egui::Color32;
use eframe::egui::{self, TextEdit};
use egui_plot::PlotPoint;
use egui_plot::{Bar, BarChart, Legend, Plot, Text};
use jobtracker_core::{Job, JobStatus, JobStore};
use std::collections::HashMap;
use strum::IntoEnumIterator;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    let mut job_app = JobApp::default();
    let _ = job_app.store.list_jobs().unwrap();

    // Tinkering with new stacked graphing logic
    // TODO this needs to be the set of Jobs for each day
    // ex) 9/9/25 = [Job1, Job2,...] all jobs whose timestamp fall in this day
    job_app.data = vec![
        vec![30.0, 45.0, 25.0, 60.0],
        vec![20.0, 35.0, 40.0, 30.0],
        vec![15.0, 25.0, 20.0, 40.0],
    ];
    job_app.group_names = vec![
        "Q1".to_string(),
        "Q2".to_string(),
        "Q3".to_string(),
        "Q4".to_string(),
    ];
    job_app.category_names = vec![
        "Product A".to_string(),
        "Product B".to_string(),
        "Product C".to_string(),
    ];
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
    new_role_location: String,
    search_text: String,
    edit_timestamps: std::collections::HashMap<u32, String>,

    // Tinkering with new stacked graphing logic
    pub data: Vec<Vec<f64>>,
    pub group_names: Vec<String>,
    pub category_names: Vec<String>,
}

impl eframe::App for JobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Job Application Tracker");
            ui.separator();

            ui.horizontal(|ui| {
                // Search box
                ui.label("Search:");
                ui.add(TextEdit::singleline(&mut self.search_text));

                // Refresh button
                if ui.add(egui::Button::new("Refresh")).clicked() {
                    let _ = self.store.list_jobs();
                }
            });

            ui.separator();

            // ----------------------------
            // Add new job form
            // ----------------------------
            ui.horizontal(|ui| {
                let text_width = 150.0; // pick a consistent width

                ui.label("Company:");
                ui.add_sized(
                    [text_width, 20.0],
                    TextEdit::singleline(&mut self.new_company),
                );

                ui.label("Role:");
                ui.add_sized([text_width, 20.0], TextEdit::singleline(&mut self.new_role));

                ui.label("Location:");
                ui.add_sized(
                    [text_width, 20.0],
                    TextEdit::singleline(&mut self.new_role_location),
                );

                if ui.button("Add").clicked()
                    && !self.new_company.is_empty()
                    && !self.new_role.is_empty()
                    && !self.new_role_location.is_empty()
                {
                    self.store
                        .add_job(
                            self.new_company.clone(),
                            self.new_role.clone(),
                            self.new_role_location.clone(),
                        )
                        .unwrap();
                    self.new_company.clear();
                    self.new_role.clear();
                    self.new_role_location.clear();
                }
            });

            ui.separator();
            // ----------------------------
            // Summary stats
            // ----------------------------
            let total_jobs = self.store.jobs.len();

            let rejected_jobs = self
                .store
                .jobs
                .iter()
                .filter(|job| job.status == JobStatus::Rejected)
                .count();

            let in_progress_jobs = self
                .store
                .jobs
                .iter()
                .filter(|job| {
                    job.status == JobStatus::Applied || job.status == JobStatus::Interview
                })
                .count();

            let job_offers = self
                .store
                .jobs
                .iter()
                .filter(|job| job.status == JobStatus::Offer)
                .count();

            ui.horizontal(|ui| {
                ui.label(format!("Total Applications: {}", total_jobs));
                ui.add_space(20.0);
                ui.label(format!("In progress: {}", in_progress_jobs));
                ui.add_space(20.0);
                ui.label(format!("Rejected: {}", rejected_jobs));
                ui.add_space(20.0);
                ui.label(format!("Offers: {}", job_offers));
            });

            ui.separator();

            // ----------------------------
            // Weekly bar chart (last 7 days)
            // ----------------------------
            let today = Utc::now();
            let last_7_days: Vec<DateTime<Utc>> = (0..10)
                .rev() // oldest day first
                .map(|i| today - Duration::days(i))
                .collect();

            // Group jobs by date (YYYY-MM-DD)
            let mut date_to_jobs: HashMap<NaiveDate, Vec<Job>> = HashMap::new();
            for job in &self.store.jobs {
                let job_date = job.timestamp.date_naive();

                // Check if this job is within the last 7 days
                let is_within_last_7_days = last_7_days.iter().any(|&ts| {
                    let ts_date = ts.date_naive();
                    job_date == ts_date // Exact date match for last 7 days
                });

                if is_within_last_7_days {
                    date_to_jobs.entry(job_date).or_default().push(job.clone());
                }
            }

            // Sort dates to ensure proper ordering on x-axis
            let mut sorted_dates: Vec<NaiveDate> = date_to_jobs.keys().cloned().collect();
            sorted_dates.sort();

            ui.label("# of Applications (Last 7 Days):");
            Plot::new("applications_chart")
                .legend(Legend::default())
                .view_aspect(2.0)
                .include_x(-0.5)
                .include_x(sorted_dates.len() as f64 - 0.5)
                .include_y(0.0)
                .show_grid(true)
                .height(250.0)
                .show(ui, |plot_ui| {
                    for (date_idx, date) in sorted_dates.iter().enumerate() {
                        if let Some(jobs) = date_to_jobs.get(date) {
                            let x_position = date_idx as f64;

                            // Create a bar for this date with height = number of jobs
                            for (k, j) in jobs.iter().enumerate() {
                                let bar = Bar::new(x_position, 1 as f64)
                                    .width(0.8)
                                    .base_offset(k as f64) // offset to stack values
                                    .fill(j.get_status_color_mapping())
                                    .name(date.format("%Y-%m-%d").to_string());
                                plot_ui.bar_chart(BarChart::new(vec![bar]));
                            }

                            // Add date label below the bar
                            plot_ui.text(
                                Text::new(
                                    PlotPoint::new(x_position, -1.0),
                                    date.format("%m/%d").to_string(),
                                )
                                .color(Color32::GRAY)
                                .anchor(egui::Align2::CENTER_TOP),
                            );
                        }
                    }

                    // Add y-axis labels for count
                    if let Some(max_jobs) = date_to_jobs.values().map(|v| v.len()).max() {
                        for count in (0..=max_jobs).step_by(if max_jobs > 10 { 2 } else { 1 }) {
                            plot_ui.text(
                                Text::new(PlotPoint::new(-0.5, count as f64), count.to_string())
                                    .color(Color32::GRAY)
                                    .anchor(egui::Align2::RIGHT_CENTER),
                            );
                        }
                    }
                });
            ui.separator();

            // ----------------------------
            // Scrollable job list grid
            // ----------------------------
            let mut to_remove: Option<usize> = None;
            let mut to_update_status: Option<(u32, JobStatus)> = None;
            let mut to_update_timestamp: Option<(u32, chrono::DateTime<chrono::Local>)> = None;

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("jobs_grid").striped(true).show(ui, |ui| {
                        let col_widths = [50.0, 180.0, 120.0, 120.0, 100.0, 100.0, 60.0];

                        // Header row
                        ui.add_sized([col_widths[0], 20.0], egui::Label::new("ID"));
                        ui.add_sized([col_widths[1], 20.0], egui::Label::new("Date Applied"));
                        ui.add_sized([col_widths[2], 20.0], egui::Label::new("Company"));
                        ui.add_sized([col_widths[3], 20.0], egui::Label::new("Role"));
                        ui.add_sized([col_widths[4], 20.0], egui::Label::new("Location"));
                        ui.add_sized([col_widths[5], 20.0], egui::Label::new("Status"));
                        ui.add_sized([col_widths[6], 20.0], egui::Label::new("Action"));
                        ui.end_row();

                        // Rows
                        let search_text = self.search_text.to_lowercase();
                        for (i, job) in self
                            .store
                            .jobs
                            .iter()
                            .filter(|job| {
                                search_text.is_empty()
                                    || job.company.to_lowercase().contains(&search_text)
                                    || job.role.to_lowercase().contains(&search_text)
                                    || job.status.to_string().to_lowercase().contains(&search_text)
                                    || job
                                        .role_location
                                        .clone()
                                        .unwrap_or_default()
                                        .to_lowercase()
                                        .contains(&search_text)
                            })
                            .enumerate()
                        {
                            ui.add_sized(
                                [col_widths[0], 20.0],
                                egui::Label::new(job.id.to_string()),
                            );

                            // ---- Editable timestamp ----
                            let ts_entry =
                                self.edit_timestamps.entry(job.id).or_insert_with(|| {
                                    job.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
                                });

                            let response =
                                ui.add_sized([col_widths[1], 20.0], TextEdit::singleline(ts_entry));

                            let pressed_enter = response.has_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));

                            if response.lost_focus() || pressed_enter {
                                if let Ok(parsed) =
                                    NaiveDateTime::parse_from_str(ts_entry, "%Y-%m-%d %H:%M:%S")
                                {
                                    if let chrono::LocalResult::Single(local_dt) =
                                        Local.from_local_datetime(&parsed)
                                    {
                                        // defer update
                                        to_update_timestamp = Some((job.id, local_dt));
                                    }
                                }
                            }

                            // ---- Company / Role / Location ----
                            ui.add_sized([col_widths[2], 20.0], egui::Label::new(&job.company));
                            ui.add_sized([col_widths[3], 20.0], egui::Label::new(&job.role));
                            ui.add_sized(
                                [col_widths[4], 20.0],
                                egui::Label::new(
                                    job.role_location.clone().unwrap_or("N/A".to_string()),
                                ),
                            );

                            // ---- Status dropdown ----
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
                                            to_update_status = Some((job.id, status));
                                        }
                                    }
                                });

                            // ---- Delete button ----
                            if ui.button("Delete").clicked() {
                                to_remove = Some(i);
                            }

                            ui.end_row();
                        }
                    });
                });

            // ----------------------------
            // Apply updates
            // ----------------------------
            if let Some((id, new_status)) = to_update_status {
                self.store.update_status(id, new_status).unwrap();
            }
            if let Some((id, new_ts)) = to_update_timestamp {
                self.store.update_timestamp(id, new_ts.into()).unwrap();

                // update the edit buffer so it shows canonical formatting
                if let Some(ts_text) = self.edit_timestamps.get_mut(&id) {
                    *ts_text = new_ts.format("%Y-%m-%d %H:%M:%S").to_string();
                }
            }
            if let Some(index) = to_remove {
                self.store.delete_job(index).unwrap();
            }
        });
    }
}
