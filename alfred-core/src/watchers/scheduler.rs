use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use crate::core::alfred::Alfred;
use crate::core::orchestrator::Orchestrator;
use crate::services::notification_store::NotificationStore;
use crate::watchers::watcher::Watcher;

pub struct WatcherScheduler;

impl WatcherScheduler {
    pub fn start(
        alfred: Arc<Mutex<Alfred>>,
        watchers: Vec<Box<dyn Watcher + Send + Sync>>,
        notifications: NotificationStore,
        interval_secs: u64,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;

                for watcher in &watchers {
                    let events = match watcher.check().await {
                        Ok(events) => events,
                        Err(e) => {
                            eprintln!("[WATCHER ERROR][{}] {}", watcher.name(), e);
                            continue;
                        }
                    };

                    for event in events {
                        let prompt = format!(
                            "AUTOMATED OBSERVATION (source: {}).\n\n\
                            [EVENT CONTEXT]\n{}\n\n\
                            [SPECIFIC OBSERVER INSTRUCTIONS]\n{}\n\n\
                            [GENERAL PROCESSING GUIDELINES]\n\
                            1. Analyze the event above. Feel free to use your investigative tools (calendar, memory, notes, etc.) to verify whether this \
                            event requires your attention or action.\n\
                            2. If concrete action is required (e.g., replying, blocking a time slot), carry it out using the appropriate tool.\n\
                            3. If no action is necessary after verification, do not trigger any unnecessary action tools.\n\
                            4. ALWAYS conclude with a brief response (1–2 sentences max) summarizing what you have verified, cross-referenced, or accomplished, \
                            to serve as a notification to the user. Example:\n\
                            'Team seminar on August 28: Confirmation draft prepared (lunch attendance included). Awaiting your instruction to send.'",
                            event.source, event.context, watcher.prompt()
                        );

                        let mut watcher_alfred = {
                            let alfred_guard = alfred.lock().await;
                            alfred_guard.new_isolated()
                        };
                        let summary = Orchestrator::ask_alfred(&prompt, &mut watcher_alfred).await;

                        /*
                        Push notification only if the word "ras" is not present
                         */
                        let clean_summary = summary.trim();
                        if !clean_summary.is_empty() && !clean_summary.eq_ignore_ascii_case("RAS") && !clean_summary.contains("RAS") {
                            notifications.push(&event.source, clean_summary).await;
                        }
                    }
                }
            }
        });
    }
}