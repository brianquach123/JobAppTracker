use jobtracker_core::JobStore;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut store = JobStore::default();

    if args.len() > 2 && args[1] == "add" {
        let company = args[2].clone();
        let role = args.get(3).unwrap_or(&"Unknown".to_string()).clone();
        let location = args.get(4).unwrap_or(&"Unknown".to_string()).clone();
        let _ = store.add_job(company, role, location);
        println!("Job added!");
    }

    if args.len() > 1 && args[1] == "list" {
        for (i, job) in store.list_jobs().unwrap().into_iter().enumerate() {
            println!("{}: {} - {} [{}]", i, job.company, job.role, job.status);
        }
    }
}
