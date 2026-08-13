use chrono::Utc;
use chrono_tz::Europe::Paris;
use crate::services::calendar::CalendarService;
use crate::services::gmail::GmailService;
use crate::services::weather::WeatherService;

#[derive(Clone)]
pub struct MorningBriefingService {
    gmail: GmailService,
    calendar: CalendarService
}

impl MorningBriefingService {

    pub fn new(gmail: GmailService, calendar: CalendarService) -> Self {
        MorningBriefingService { gmail, calendar}
    }

    pub async fn generate_briefing(&self) -> Result<String, String>{
        //Recover daily events
        let events = self.calendar.list_upcoming(10).await.unwrap_or_default();
        let events_text = if events.is_empty() {
            String::from("No events")
        }
        else{
            events
                .iter()
                .map(|e| format!("- {} (from {} to {})", e.summary, e.start, e.end))
                .collect::<Vec<_>>()
                .join("\n")
        };

        //Recover mails
        let mails = self.gmail.list_unread(5).await.unwrap_or_default();
        let mails_text = if mails.is_empty() {
            "No urgent messages.".to_string()
        } else {
            mails
                .iter()
                .map(|e| format!("- De {}: {}", e.from, e.subject))
                .collect::<Vec<_>>()
                .join("\n")
        };

        //Recover day and time
        let now_paris = Utc::now().with_timezone(&Paris);
        let date_str = now_paris.format("%Y-%m-%d (%A %d %B %Y)").to_string();
        let time_str = now_paris.format("%H:%M").to_string();

        //Generate prompt
        let prompt = format!(
            "RÔLE : Tu es Alfred, l'assistant personnel et majordome de l'utilisateur. C'est son réveil.

            CONTEXTE TEMPOREL STRICT :
            - Aujourd'hui nous sommes le : {}
            - Heure actuelle : {} (Fuseau : Europe/Paris)

            DONNÉES BRUTES DE L'AGENDA :
            [Agenda]
            {}

            [Messages importants]
            {}

            CONSIGNES STRICTES :
            1. Compare la date de chaque événement avec la date d'aujourd'hui ({}).
               - Si un événement a lieu un autre jour (comme demain), NE DIS PAS qu'il a lieu aujourd'hui.
            2. Salue l'utilisateur chaleureusement, souhaite-lui un bon réveil et résume uniquement sa journée d'aujourd'hui en 3 à 4 phrases maximum.
            3. Termine par une note d'encouragement ou une phrase élégante de majordome.",
            date_str, time_str, events_text, mails_text, date_str
        );

        Ok(prompt)
    }
}