use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Applied,
    Interview,
    Offer,
    Rejected,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub company: String,
    pub role: String,
    pub status: JobStatus,
    pub timestamp: DateTime<Utc>,
}

impl Job {
    pub fn new(company: String, role: String) -> Self {
        Self {
            company,
            role,
            status: JobStatus::Applied,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Default)]
pub struct JobStore {
    pub jobs: Vec<Job>,
}

impl JobStore {
    pub fn add_job(&mut self, company: String, role: String) {
        self.jobs.push(Job::new(company, role));
    }

    pub fn list_jobs(&self) -> &[Job] {
        &self.jobs
    }

    pub fn delete_job(&mut self, index: usize) {
        if index < self.jobs.len() {
            self.jobs.remove(index);
        }
    }

    pub fn update_status(&mut self, index: usize, new_status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(index) {
            job.status = new_status;
        }
    }
}
