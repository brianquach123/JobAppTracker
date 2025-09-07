use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub company: String,
    pub role: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

impl Job {
    pub fn new(company: String, role: String) -> Self {
        Self {
            company,
            role,
            status: "Applied".to_string(),
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

    pub fn update_status(&mut self, index: usize, new_status: String) {
        if let Some(job) = self.jobs.get_mut(index) {
            job.status = new_status;
        }
    }
}
