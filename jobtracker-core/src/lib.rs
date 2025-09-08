use anyhow::Error;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use strum_macros::EnumIter;

const FILE: &str = "jobtrack.json";

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
    pub id: u32,
    pub company: String,
    pub role: String,
    pub role_location: String,
    pub status: JobStatus,
    pub timestamp: DateTime<Utc>,
}

impl Job {
    pub fn new(id: u32, company: String, role: String, new_role_location: String) -> Self {
        Self {
            id,
            company,
            role,
            role_location: new_role_location,
            status: JobStatus::Applied,
            timestamp: Utc::now(),
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
            Ok(vec![])
        } else {
            Ok(serde_json::from_str(&data)?)
        }
    } else {
        Ok(vec![])
    }
}

#[derive(Default)]
pub struct JobStore {
    pub jobs: Vec<Job>,
}

impl JobStore {
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
}
