use chrono::Utc;
use chrono_tz::Europe::Paris;

/*
Recover Paris time
 */
pub fn handle_time() -> String {
    let paris_time = Utc::now().with_timezone(&Paris);
    return paris_time.format("%H:%M:%S").to_string()
}