use axum::{extract::State, Json};
use alfred_core::services::notification_store::{NotificationStore, Notification};

pub async fn notifications_handle(State(store): State<NotificationStore>) -> Json<Vec<Notification>> {
    Json(store.drain().await)
}