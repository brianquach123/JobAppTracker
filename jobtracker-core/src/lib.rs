mod job;
mod job_app;
mod job_store;
mod summary_counts;
use anyhow::Result;
use chrono::{DateTime, Utc};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
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

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Applied => {
                write!(f, "Applied")
            }
            JobStatus::Interview => {
                write!(f, "Interview")
            }
            JobStatus::Offer => {
                write!(f, "Offer")
            }
            JobStatus::Rejected => {
                write!(f, "Rejected")
            }
            JobStatus::Ghosted => {
                write!(f, "Ghosted")
            }
        }
    }
}

#[derive(Default, EnumIter, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JobSource {
    Recruiter,
    LinkedIn,
    Monster,
    Indeed,
    #[default]
    NotProvided,
}

impl fmt::Display for JobSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobSource::LinkedIn => write!(f, "LinkedIn"),
            JobSource::Monster => write!(f, "Monster"),
            JobSource::Indeed => write!(f, "Indeed"),
            JobSource::Recruiter => write!(f, "Recruiter"),
            JobSource::NotProvided => write!(f, "Not provided"),
        }
    }
}

impl FromStr for JobSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "linkedIn" => Ok(JobSource::LinkedIn),
            "monster" => Ok(JobSource::Monster),
            "indeed" => Ok(JobSource::Indeed),
            "recruiter" => Ok(JobSource::Recruiter),
            _ => Ok(JobSource::NotProvided),
        }
    }
}
