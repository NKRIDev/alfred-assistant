use std::thread;
use std::time::Duration;
use crate::services::memory::MemoryService;

/*
Task that updates the assistant's memory
and restarts it.
 */
pub struct DreamTimer;

impl DreamTimer {

    /*
    Start timer with database path url and
    timer in secs.
     */
    pub fn start(database: String, timer: u64){
        thread::spawn(move || {
            let memory_service = MemoryService::new(&database)
                .expect("Error opening memory database.");

            loop {
                thread::sleep(Duration::from_secs(timer));

                println!("[SYSTEM] Auto Dream process started.");
                println!("{}", memory_service.dream());
                println!("[SYSTEM] Auto Dream cycle finished successfully.");
            }
        });
    }
}