use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use eframe::egui::{self, Align, Layout, TextEdit, Ui};
use eframe::egui::{Color32, Stroke};
use egui_plot::PlotPoint;
use egui_plot::{Bar, BarChart, Legend, Plot, Text};
use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::{Job, JobApp, JobSource, JobStatus};

pub const DEFAULT_FIELD_ELEMENT_HEIGHT: f32 = 20.0;
pub const COLUMN_HEADER_AND_WIDTH_FIELDS: [(&str, f32); 8] = [
    ("ID", 50.0),
    ("Date Applied", 180.0),
    ("Company", 120.0),
    ("Role", 120.0),
    ("Location", 100.0),
    ("Status", 100.0),
    ("Source", 60.0),
    ("Action", 60.0),
];

impl JobApp {
    fn add_search_box(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(
                TextEdit::singleline(&mut self.search_text)
                    .desired_width(ui.available_width() * 0.3),
            );
        });
    }

    fn add_refresh_button(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Refresh")).clicked() {
                let _ = self.store.list_jobs();
                self.last_refresh = Utc::now();
            }
            ui.label(format!(
                "Last Refresh: {}",
                self.last_refresh
                    .with_timezone(&New_York)
                    .format("%Y-%m-%d %H:%M:%S")
            ));
        });
    }

    fn add_job_app_input_form(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.vertical(|ui| {
                let label_width = 80.0;
                let field_width = (ui.available_width() / 2.0) - label_width - 10.0;

                ui.horizontal(|ui| {
                    ui.add_sized([label_width, 20.0], egui::Label::new("Company:"));
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_company),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_sized([label_width, 20.0], egui::Label::new("Role:"));
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_role),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_sized([label_width, 20.0], egui::Label::new("Location:"));
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_role_location),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_sized([label_width, 20.0], egui::Label::new("Source:"));
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_source),
                    );
                });

                if ui.button("Add").clicked()
                    && !self.new_company.is_empty()
                    && !self.new_role.is_empty()
                    && !self.new_role_location.is_empty()
                    && !self.new_source.is_empty()
                {
                    self.store
                        .add_job(
                            self.new_company.clone(),
                            self.new_role.clone(),
                            self.new_role_location.clone(),
                            self.new_source.clone(),
                        )
                        .unwrap();
                    self.new_company.clear();
                    self.new_role.clear();
                    self.new_role_location.clear();
                }
            });
        });
    }

    fn add_bar_chart_stats(&mut self, ui: &mut Ui) {
        self.store.calculate_summary_stats().unwrap();

        // Find earliest application date (fallback: today if no jobs yet)
        let today = Utc::now();
        let earliest_date = self
            .store
            .jobs
            .iter()
            .map(|job| job.timestamp.date_naive())
            .min()
            .unwrap_or_else(|| today.date_naive());

        // Collect every day from earliest_date..=today
        let all_dates: Vec<NaiveDate> = {
            let mut dates = Vec::new();
            let mut d = earliest_date;
            while d <= today.date_naive() {
                dates.push(d);
                d = d.succ_opt().unwrap(); // go to next day safely
            }
            dates
        };

        // Initialize the map with empty vectors
        let mut date_to_jobs: HashMap<NaiveDate, Vec<Job>> =
            all_dates.iter().map(|&d| (d, Vec::new())).collect();

        // Assign jobs to their dates
        for job in &self.store.jobs {
            let job_date = job.timestamp.date_naive();
            if date_to_jobs.contains_key(&job_date) {
                date_to_jobs.get_mut(&job_date).unwrap().push(job.clone());
            }
        }

        // Sorted list of dates for x-axis
        let mut sorted_dates: Vec<NaiveDate> = date_to_jobs.keys().cloned().collect();
        sorted_dates.sort();

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            let padding = " ".repeat(20);
            ui.label(format!(
                "Timeline:\n\nRejection: {:.2}%{padding}Interview: {:.2}%",
                (self.store.summary_stats.rejected as f32 / self.store.summary_stats.total as f32)
                    * 100.0,
                (self.store.summary_stats.interviews as f32
                    / self.store.summary_stats.total as f32)
                    * 100.0
            ));

            Plot::new("applications_chart")
                .legend(Legend::default())
                .include_y(0.0)
                .show_grid(true)
                .height(250.0)
                .show(ui, |plot_ui| {
                    for (date_idx, date) in sorted_dates.iter().enumerate() {
                        if let Some(jobs) = date_to_jobs.get(date) {
                            let x_position = date_idx as f64;

                            // Create a bar for this date with height = number of jobs
                            for (k, j) in jobs.iter().enumerate() {
                                let is_selected =
                                    self.selected_company.as_ref() == Some(&j.company);
                                let stroke = if is_selected {
                                    Stroke::new(3.0, Color32::GOLD) // thicker border
                                } else {
                                    Stroke::new(0.3, Color32::BLACK) // normal border
                                };

                                if self.search_text.is_empty() {
                                    self.selected_company = None;
                                }

                                let bar = Bar::new(x_position, 1_f64)
                                    .width(0.8)
                                    .base_offset(k as f64) // offset to stack values
                                    .fill(j.get_status_color_mapping())
                                    .stroke(stroke)
                                    .name(format!("{}\n{}", j.company, j.role));
                                plot_ui.bar_chart(BarChart::new(vec![bar]));
                            }

                            // Add date label below the bar
                            if date_idx % 4 == 0 {
                                plot_ui.text(
                                    Text::new(
                                        PlotPoint::new(x_position, -0.5),
                                        date.format("%m/%d").to_string(),
                                    )
                                    .color(Color32::GRAY)
                                    .anchor(egui::Align2::CENTER_TOP),
                                );
                            }
                        }
                    }

                    // Selectable chart entries that'll dynamically search for the app clicked
                    if plot_ui.response().clicked() {
                        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                            let x_idx = pointer_pos.x.round() as usize;
                            if let Some(date) = sorted_dates.get(x_idx) {
                                if let Some(jobs) = date_to_jobs.get(date) {
                                    // Find the "stack level" based on y coordinate
                                    let stack_idx = pointer_pos.y.floor() as usize;
                                    if let Some(job) = jobs.get(stack_idx) {
                                        // Update search text to clicked company
                                        self.search_text = job.company.clone();
                                        self.selected_company = Some(job.company.clone());
                                    }
                                }
                            }
                        }
                    }
                });
        });
        // ----------------------------
        // Bar Chart Legend
        // ----------------------------
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.columns(5, |columns| {
                columns[0].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(65, 105, 225),
                        );
                        ui.add_space(20.0);
                        ui.label(format!("Applied: {}", self.store.summary_stats.total));
                        ui.add_space(10.0);
                    });
                });
                columns[1].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(0, 255, 255),
                        );
                        ui.add_space(20.0);
                        ui.label(format!(
                            "Interview: {}",
                            self.store.summary_stats.interviews
                        ));
                        ui.add_space(10.0);
                    });
                });
                columns[2].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(0, 255, 0),
                        );
                        ui.add_space(20.0);
                        ui.label(format!("Offer: {}", self.store.summary_stats.offers));
                    });
                });
                columns[3].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(255, 0, 0),
                        );
                        ui.add_space(20.0);
                        ui.label(format!("Rejected: {}", self.store.summary_stats.rejected));
                        ui.add_space(10.0);
                    });
                });
                columns[4].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(128, 128, 128),
                        );
                        ui.add_space(20.0);
                        ui.label(format!("Ghosted: {}", self.store.summary_stats.ghosted));
                        ui.add_space(10.0);
                    });
                });
            });
        });
    }
}

impl eframe::App for JobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_bar_chart_stats(ui);
            ui.separator();

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    self.add_job_app_input_form(ui);
                    ui.vertical(|ui| {
                        self.add_search_box(ui);
                        self.add_refresh_button(ui);
                    });
                });
            });
            ui.separator();

            // ----------------------------
            // Scrollable job list grid
            // ----------------------------
            let mut to_remove: Option<usize> = None;
            let mut to_update_status: Option<(u32, JobStatus)> = None;
            let mut to_update_source: Option<(u32, JobSource)> = None;
            let mut to_update_timestamp: Option<(u32, chrono::DateTime<chrono::Local>)> = None;
            let mut to_update_company: Option<(u32, String)> = None;

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("jobs_grid").striped(true).show(ui, |ui| {
                        // Header row
                        for (idx, header_field) in COLUMN_HEADER_AND_WIDTH_FIELDS.iter().enumerate()
                        {
                            ui.add_sized(
                                [header_field.1, DEFAULT_FIELD_ELEMENT_HEIGHT],
                                egui::Label::new(COLUMN_HEADER_AND_WIDTH_FIELDS[idx].0),
                            );
                        }
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
                                [50.0, DEFAULT_FIELD_ELEMENT_HEIGHT],
                                egui::Label::new(job.id.to_string()),
                            );

                            // ---- Editable timestamp ----
                            let ts_entry =
                                self.edit_timestamps.entry(job.id).or_insert_with(|| {
                                    job.timestamp
                                        .with_timezone(&Local)
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string()
                                });

                            let response = ui.add_sized(
                                [
                                    COLUMN_HEADER_AND_WIDTH_FIELDS[1].1,
                                    DEFAULT_FIELD_ELEMENT_HEIGHT,
                                ],
                                TextEdit::singleline(ts_entry),
                            );

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
                            let curr_company = self
                                .edit_companies
                                .entry(job.id)
                                .or_insert_with(|| job.company.clone());

                            let response = ui.add_sized(
                                [
                                    COLUMN_HEADER_AND_WIDTH_FIELDS[2].1,
                                    DEFAULT_FIELD_ELEMENT_HEIGHT,
                                ],
                                TextEdit::singleline(curr_company),
                            );
                            let pressed_enter = response.has_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if response.lost_focus() || pressed_enter {
                                to_update_company = Some((job.id, curr_company.to_string()));
                            }

                            ui.add_sized(
                                [
                                    COLUMN_HEADER_AND_WIDTH_FIELDS[3].1,
                                    DEFAULT_FIELD_ELEMENT_HEIGHT,
                                ],
                                egui::Label::new(&job.role),
                            );
                            ui.add_sized(
                                [
                                    COLUMN_HEADER_AND_WIDTH_FIELDS[4].1,
                                    DEFAULT_FIELD_ELEMENT_HEIGHT,
                                ],
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

                            // Source
                            let mut selected_source =
                                job.source.as_ref().unwrap_or(&JobSource::LinkedIn).clone();
                            egui::ComboBox::from_id_source(format!("source_{}", i))
                                .selected_text(selected_source.to_string())
                                .show_ui(ui, |ui| {
                                    for src in JobSource::iter() {
                                        if ui
                                            .selectable_value(
                                                &mut selected_source,
                                                src.clone(),
                                                src.to_string(),
                                            )
                                            .clicked()
                                        {
                                            to_update_source = Some((job.id, src));
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
            if let Some((id, new_source)) = to_update_source {
                self.store.update_source(id, new_source).unwrap();
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
            if let Some((id, new_company)) = to_update_company {
                self.store.update_company(id, new_company.clone()).unwrap();

                // update the edit buffer so it shows canonical formatting
                if let Some(company) = self.edit_companies.get_mut(&id) {
                    *company = new_company;
                }
            }
        });
    }
}
