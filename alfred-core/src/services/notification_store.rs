use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;
use chrono::Utc;

/*
System notification, to be sent to the frontend
 */
#[derive(Serialize, Clone)]
pub struct Notification {
    pub id: String,
    pub source: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct NotificationStore {
    items: Arc<Mutex<Vec<Notification>>>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self { items: Arc::new(Mutex::new(Vec::new())) }
    }

    pub async fn push(&self, source: &str, message: &str) {
        let mut items = self.items.lock().await;
        items.push(Notification {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_string(),
            message: message.to_string(),
            created_at: Utc::now().to_rfc3339(),
        });
    }

    /*
    Consumes pending notifications
     */
    pub async fn drain(&self) -> Vec<Notification> {
        let mut items = self.items.lock().await;
        std::mem::take(&mut *items)
    }
}