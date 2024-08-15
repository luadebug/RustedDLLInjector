use libmem::process::{enum_processes, Process};
use tracing::error;

//info

pub fn get_process_list() -> Vec<Process> {
    match enum_processes() {
        None => {
            error!("Failed to get process list");
            Vec::new()
        }
        Some(processlist) => {
            //info!("Got process list: {:?}", processlist);
            processlist
        }
    }
}
