use crate::JobSource;
use anyhow::Result;
use std::fmt;
use std::str::FromStr;

impl fmt::Display for JobSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobSource::LinkedIn => write!(f, "LinkedIn"),
            JobSource::Monster => write!(f, "Monster"),
            JobSource::Indeed => write!(f, "Indeed"),
            JobSource::Recruiter => write!(f, "Recruiter"),
            JobSource::NotProvided => write!(f, "Not provided"),
            JobSource::Talent => write!(f, "Talent.com"),
            JobSource::Glassdoor => write!(f, "Glassdoor"),
            JobSource::ZipRecruiter => write!(f, "ZipRecruiter"),
        }
    }
}

impl FromStr for JobSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "linkedin" => Ok(JobSource::LinkedIn),
            "monster" => Ok(JobSource::Monster),
            "indeed" => Ok(JobSource::Indeed),
            "recruiter" => Ok(JobSource::Recruiter),
            "talent.com" => Ok(JobSource::Talent),
            "glassdoor" => Ok(JobSource::Glassdoor),
            "ziprecruiter" => Ok(JobSource::ZipRecruiter),
            _ => Ok(JobSource::NotProvided),
        }
    }
}
