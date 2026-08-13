use async_trait::async_trait;
use crate::services::calendar::CalendarService;
use crate::watchers::watcher::{Watcher, WatcherEvent};

pub struct CalendarWatcher {
    calendar_service: CalendarService,
}

impl CalendarWatcher {
    pub fn new(calendar_service: CalendarService) -> Self {
        Self { calendar_service }
    }
}

#[async_trait]
impl Watcher for CalendarWatcher {
    fn name(&self) -> String {
        String::from("calendar")
    }

    async fn check(&self) -> Result<Vec<WatcherEvent>, String> {
        //Retrieve events for the next 24 or 48 hours
        let events = match self.calendar_service.list_upcoming(10).await {
            Ok(events) => events,
            Err(e) => return Err(format!("[CALENDAR WATCHER] Error reading the calendar : {}", e)),
        };

        if events.is_empty() {
            print!("[CALENDAR WATCHER] No events to watch");
            return Ok(vec![]);
        }

        //Formate events to string
        let formatted_events = events
            .iter()
            .map(|e| {
                format!(
                    "- [{}] {} (Start: {}, End: {})",
                    e.id, e.summary, e.start, e.end
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let events_summary = format!(
            "Here are the upcoming events listed in the calendar. :\n\n{}",
            formatted_events
        );

        //Send watcher event to LLM
        Ok(vec![WatcherEvent {
            source: String::from("Google Calendar"),
            context: events_summary,
        }])
    }

    fn prompt(&self) -> String {
        "ROLE: You are Alfred. You must act like a proper butler / personal assistant.

        STRICT GUIDELINES:
        1. Review the incoming events. Your role is SOLELY to:
        - Detect any event starting VERY SOON (within the next 6 hours) to issue a brief reminder.
        - Detect any strict CONFLICT OR OVERLAP between two events.

        2. SILENCE RULE (CRITICAL):
        - If all events are routine, not imminent, and conflict-free, REPLY ONLY WITH THE WORD: 'RAS'.
        - Do NOT make comments like 'Everything is fine', 'Clear schedule', or 'No overlaps'. Remain silent.

        3. REMINDER FORMAT (If applicable)
        - Be extremely concise (1 sentence).
        - Example: 'Reminder: Your meeting with Polar starts at 2:00 PM.'
        - Conflict example: 'Alert: Conflict at 2:00 PM between the Polar appointment and the Client Meeting.'".to_string()
    }
}