use anyhow::Error;
use anyhow::Result;
use chrono::{DateTime, Utc};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use strum_macros::EnumIter;

pub const APP_NAME: &str = "Job Application Tracker";
pub const WINDOW_WIDTH: f32 = 1200.0;
pub const WINDOW_HEIGHT: f32 = 800.0;
pub const DEFAULT_FIELD_ELEMENT_HEIGHT: f32 = 20.0;
pub const COLUMN_HEADER_AND_WIDTH_FIELDS: [(&str, f32); 8] = [
    ("ID", 50.0),
    ("Date Applied", 180.0),
    ("Company", 120.0),
    ("Role", 120.0),
    ("Location", 100.0),
    ("Status", 100.0),
    ("Action", 60.0),
    ("Source", 60.0),
];

const FILE: &str = "jobtrack.json";
const NAVY_BLUE: Color32 = Color32::from_rgb(65, 105, 225);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const GREEN: Color32 = Color32::from_rgb(0, 255, 0);
const RED: Color32 = Color32::from_rgb(255, 0, 0);
const GRAY: Color32 = Color32::from_rgb(128, 128, 128);

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
}

impl Job {
    pub fn new(id: u32, company: String, role: String, new_role_location: String) -> Self {
        Self {
            id,
            company,
            role,
            role_location: Some(new_role_location),
            status: JobStatus::Applied,
            timestamp: Utc::now(),
        }
    }

    pub fn get_status_color_mapping(&self) -> Color32 {
        match self.status {
            JobStatus::Applied => NAVY_BLUE,
            JobStatus::Interview => CYAN,
            JobStatus::Offer => GREEN,
            JobStatus::Rejected => RED,
            JobStatus::Ghosted => GRAY,
        }
    }
}

pub fn save(apps: &[Job]) -> Result<()> {
    let data = serde_json::to_string_pretty(apps)?;
    fs::write(FILE, data)?;
    Ok(())
}

pub fn load() -> Result<Vec<Job>> {
    if let Ok(mut file) = OpenOptions::new().read(true).open(FILE) {
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        if data.trim().is_empty() {
            println!("Got empty data from file");
            Ok(vec![])
        } else {
            println!("Got data, deserializing");
            Ok(serde_json::from_str(&data)?)
        }
    } else {
        println!("Error opening data file");
        Ok(vec![])
    }
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

impl fmt::Display for SummaryCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let padding = " ".repeat(20);
        write!(f, "Total Applications: {}", self.total)?;
        write!(f, "{padding}Applied: {}", self.applied)?;
        write!(f, "{padding}Rejected: {}", self.rejected)?;
        write!(f, "{padding}Ghosted: {}", self.ghosted)?;
        write!(f, "{padding}Interviews: {}", self.interviews)?;
        write!(
            f,
            "{padding}Rejection Rate: {:.2}%",
            (self.rejected as f32 / self.total as f32) * 100.0
        )?;
        write!(
            f,
            "{padding}Interview Rate: {:.2}%",
            (self.interviews as f32 / self.total as f32) * 100.0
        )
    }
}

#[derive(Default, Debug)]
pub struct JobStore {
    pub jobs: Vec<Job>,
    pub summary_stats: SummaryCounts,
}

impl JobStore {
    pub fn calculate_summary_stats(&mut self) -> Result<(), Error> {
        // TODO: Add a periodic check for this? dont need to iterate every frame.
        // Reset counts to account for the egui update() tick
        self.summary_stats = SummaryCounts::default();
        for job in &self.jobs {
            self.summary_stats.total += 1;
            match job.status {
                JobStatus::Rejected => self.summary_stats.rejected += 1,
                JobStatus::Ghosted => self.summary_stats.ghosted += 1,
                JobStatus::Applied => self.summary_stats.applied += 1,
                JobStatus::Interview => self.summary_stats.interviews += 1,
                JobStatus::Offer => self.summary_stats.offers += 1,
            }
        }
        Ok(())
    }

    pub fn add_job(
        &mut self,
        company: String,
        role: String,
        new_role_location: String,
    ) -> Result<Vec<Job>, Error> {
        let new_job_id = self.jobs.iter().map(|a| a.id).max().unwrap_or(0) + 1;
        self.jobs
            .push(Job::new(new_job_id, company, role, new_role_location));
        save(&self.jobs)?;
        Ok(self.jobs.clone())
    }

    pub fn list_jobs(&mut self) -> Result<Vec<Job>, Error> {
        self.jobs = load()?;
        Ok(self.jobs.clone())
    }

    pub fn delete_job(&mut self, index: usize) -> Result<Vec<Job>, Error> {
        if index < self.jobs.len() {
            self.jobs.remove(index);
            save(&self.jobs)?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_status(&mut self, id: u32, new_status: JobStatus) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = new_status;
            save(&self.jobs)?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_company(&mut self, id: u32, new_company: String) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.company = new_company;
            save(&self.jobs)?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_timestamp(
        &mut self,
        id: u32,
        new_timestamp: DateTime<Utc>,
    ) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.timestamp = new_timestamp;
            save(&self.jobs)?;
        }
        Ok(self.jobs.clone())
    }
}
