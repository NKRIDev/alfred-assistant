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

            loop {
                thread::sleep(Duration::from_secs(timer));

                match MemoryService::new(&database) {
                    Ok(service) => {
                        println!("[SYSTEM] Auto Dream process started.");
                        println!("{}", service.dream());
                        println!("[SYSTEM] Auto Dream cycle finished successfully.");
                    },
                    Err(e) => {
                        eprintln!("Error opening memory database: {}", e);
                        continue;
                    }
                }
            }
        });
    }
}