use std::sync::Arc;
use tokio::sync::Mutex;
use alfred_core::core::alfred::Alfred;
use alfred_core::core::orchestrator::Orchestrator;

/*
Business service: holds Alfred (shared, thread-safe)
and exposes the logic, modifying the HTTP protocol
 */
#[derive(Clone)]
pub struct AskService {
    alfred: Arc<Mutex<Alfred>>,
}

impl AskService {
    pub fn new(alfred: Arc<Mutex<Alfred>>) -> Self {
        Self { alfred }
    }

    pub async fn ask(&self, text: String) -> String {
        let mut alfred = self.alfred.lock().await;
        Orchestrator::ask_alfred(&text, &mut alfred).await
    }
}