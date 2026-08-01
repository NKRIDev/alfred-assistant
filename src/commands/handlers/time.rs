/*
Recover paris time and display this
 */

use chrono::Utc;
use chrono_tz::Europe::Paris;
use crate::commands::command::CommandHandler;

pub struct TimeCommand;

impl CommandHandler for TimeCommand {
    fn execute(&self) {
        let paris_time = Utc::now().with_timezone(&Paris);
        println!("{}", paris_time.format("%H:%M:%S").to_string());
    }
}