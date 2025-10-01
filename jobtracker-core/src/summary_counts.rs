use crate::SummaryCounts;
use std::fmt;

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
