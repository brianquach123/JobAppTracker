mod job;
mod job_app;
mod job_source;
mod job_status;
mod job_store;
mod summary_counts;
use chrono::{DateTime, Utc};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::EnumIter;

pub const APP_NAME: &str = "Job Application Tracker";
pub const WINDOW_WIDTH: f32 = 1200.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

const NAVY_BLUE: Color32 = Color32::from_rgb(65, 105, 225);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const GREEN: Color32 = Color32::from_rgb(0, 255, 0);
const RED: Color32 = Color32::from_rgb(255, 0, 0);
const GRAY: Color32 = Color32::from_rgb(128, 128, 128);

/// Representation of the application itself.
#[derive(Default)]
pub struct JobApp {
    /// Internal datastore of all job applications so far.
    pub store: JobStore,
    /// Input element in form.
    pub new_company: String,
    /// Input element in form.
    pub new_role: String,
    /// Input element in form.
    pub new_role_location: String,
    /// Input element in form
    pub new_source: String,
    /// Input element in form
    pub search_text: String,
    /// The set of timestamps the user has edited in the form.
    pub edit_timestamps: HashMap<u32, String>,
    /// The set of company names the user has edited in the form.
    pub edit_companies: HashMap<u32, String>,
    /// Last time the data file (DB TODO) was successfully read and deserialized.
    pub last_refresh: DateTime<Utc>,
    /// Tracks which chart entry the user's currently selected. This is used for
    /// highlighting and filtering for a specific job application through the stacked
    /// bar chart.
    pub selected_company: Option<String>,
}

#[derive(Default, Debug)]
pub struct JobStore {
    pub jobs: Vec<Job>,
    pub summary_stats: SummaryCounts,
}

#[derive(Default, Debug)]
pub struct SummaryCounts {
    pub total: usize,
    pub rejected: usize,
    pub ghosted: usize,
    pub applied: usize,
    pub interviews: usize,
    pub offers: usize,
}

/// Representation of a job application entered by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Sequential ID number.
    pub id: u32,
    /// Name of the place the user applied to.
    pub company: String,
    /// Name of the position the user applied to.
    pub role: String,
    /// Location of the position.
    pub role_location: Option<String>,
    /// State of this job application.
    pub status: JobStatus,
    /// When this application was entered into the tracker.
    /// This is reported in UTC in the frontend. The exact
    /// value of this is set when the user clicks the button
    /// to add a new application to the tracker.
    pub timestamp: DateTime<Utc>,
    /// Where this job application was discovered.
    pub source: Option<JobSource>,
}

/// The states a job application may be in.
/// A job application will only be in one state
/// at any moment.
#[derive(EnumIter, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JobStatus {
    Applied,
    Interview,
    Offer,
    Rejected,
    Ghosted,
}

#[derive(Default, EnumIter, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JobSource {
    Recruiter,
    LinkedIn,
    Monster,
    Indeed,
    #[default]
    NotProvided,
    Talent,
    Glassdoor,
}
