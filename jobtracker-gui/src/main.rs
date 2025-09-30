use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use chrono::{NaiveDate, Utc};
use chrono_tz::America::New_York;
use eframe::egui::{self, Align, Layout, TextEdit, Ui};
use eframe::egui::{Color32, Stroke};
use egui_plot::PlotPoint;
use egui_plot::{Bar, BarChart, Legend, Plot, Text};
use jobtracker_core::{
    Job, JobSource, JobStatus, JobStore, APP_NAME, COLUMN_HEADER_AND_WIDTH_FIELDS,
    DEFAULT_FIELD_ELEMENT_HEIGHT, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(true),
        ..Default::default()
    };

    let mut job_app = JobApp {
        last_refresh: Utc::now(),
        ..Default::default()
    };
    let _ = job_app.store.list_jobs().unwrap();

    eframe::run_native(APP_NAME, options, Box::new(|_cc| Ok(Box::new(job_app))))
}

/// Representation of the application itself.
#[derive(Default)]
struct JobApp {
    /// Internal datastore of all job applications so far.
    store: JobStore,
    /// Input element in form.
    new_company: String,
    /// Input element in form.
    new_role: String,
    /// Input element in form.
    new_role_location: String,
    /// Input element in form
    new_source: String,
    /// Input element in form
    search_text: String,
    /// The set of timestamps the user has edited in the form.
    edit_timestamps: HashMap<u32, String>,
    /// The set of company names the user has edited in the form.
    edit_companies: HashMap<u32, String>,
    /// Last time the data file (DB TODO) was successfully read and deserialized.
    last_refresh: DateTime<Utc>,
    /// Tracks which chart entry the user's currently selected. This is used for
    /// highlighting and filtering for a specific job application through the stacked
    /// bar chart.
    selected_company: Option<String>,
}

impl JobApp {
    fn add_search_box(&mut self, ui: &mut Ui) {
        ui.label("Search:");
        ui.add(
            TextEdit::singleline(&mut self.search_text).desired_width(ui.available_width() * 0.3),
        );
    }

    fn add_refresh_button(&mut self, ui: &mut Ui) {
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
    }

    fn add_job_app_input_form(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let field_width = ui.available_width() / 4.0;
                ui.horizontal(|ui| {
                    ui.label("Company:");
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_company),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Role:");
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_role),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Location:");
                    ui.add_sized(
                        [field_width, 20.0],
                        TextEdit::singleline(&mut self.new_role_location),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Source:");
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

    fn add_summary_stats(&mut self, ui: &mut Ui) {
        self.store.calculate_summary_stats().unwrap();
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.label(self.store.summary_stats.to_string());
        });
    }

    fn add_bar_chart_stats(&mut self, ui: &mut Ui) {
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
            ui.label("Application Timeline");
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
        });
        // ----------------------------
        // Bar Chart Legend
        // ----------------------------
        ui.horizontal(|ui| {
            ui.columns(4, |columns| {
                columns[0].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(16.0, 16.0)),
                            2.0,
                            Color32::from_rgb(65, 105, 225),
                        );
                        ui.add_space(20.0);
                        ui.label("Applied".to_string());
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
                        ui.label("Interview".to_string());
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
                        ui.label("Offer".to_string());
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
                        ui.label("Rejected".to_string());
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
            ui.horizontal(|ui| {
                self.add_search_box(ui);
                self.add_refresh_button(ui);
            });
            ui.separator();

            ui.horizontal(|ui| {
                self.add_job_app_input_form(ui);
            });

            ui.horizontal(|ui| {
                self.add_summary_stats(ui);
            });
            ui.separator();

            self.add_bar_chart_stats(ui);
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
                                    job.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
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
