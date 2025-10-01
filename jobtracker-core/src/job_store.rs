use crate::Job;
use crate::JobSource;
use crate::JobStatus;
use crate::JobStore;
use crate::SummaryCounts;
use anyhow::Error;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;

const FILE: &str = "jobtrack.json";

impl JobStore {
    pub fn save_to_file(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.jobs)?;
        fs::write(FILE, data)?;
        Ok(())
    }

    pub fn load_from_file(&mut self) -> Result<(), Error> {
        if let Ok(mut file) = OpenOptions::new().read(true).open(FILE) {
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            if data.trim().is_empty() {
                println!("Got empty data from file");
                Ok(())
            } else {
                println!("Got data, deserializing");
                self.jobs = serde_json::from_str(&data)?;
                Ok(())
            }
        } else {
            println!("Error opening data file");
            Ok(())
        }
    }

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
        new_source: String,
    ) -> Result<Vec<Job>, Error> {
        let new_job_id = self.jobs.iter().map(|a| a.id).max().unwrap_or(0) + 1;
        self.jobs.push(Job {
            id: new_job_id,
            company,
            role,
            role_location: Some(new_role_location),
            status: JobStatus::Applied,
            timestamp: Utc::now(),
            source: Some(new_source.parse().unwrap()),
        });
        self.save_to_file()?;
        Ok(self.jobs.clone())
    }

    pub fn list_jobs(&mut self) -> Result<Vec<Job>, Error> {
        Ok(self.jobs.clone())
    }

    pub fn delete_job(&mut self, index: usize) -> Result<Vec<Job>, Error> {
        if index < self.jobs.len() {
            self.jobs.remove(index);
            self.save_to_file()?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_status(&mut self, id: u32, new_status: JobStatus) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = new_status;
            self.save_to_file()?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_source(&mut self, id: u32, new_source: JobSource) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.source = Some(new_source);
            self.save_to_file()?;
        }
        Ok(self.jobs.clone())
    }

    pub fn update_company(&mut self, id: u32, new_company: String) -> Result<Vec<Job>, Error> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.company = new_company;
            self.save_to_file()?;
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
            self.save_to_file()?;
        }
        Ok(self.jobs.clone())
    }
}
