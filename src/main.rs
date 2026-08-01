pub mod cli;
pub mod commands;
use dotenvy::dotenv;

fn main() {
    //Init .env
    dotenv().ok();

    //Start alfred loop
    cli::start_alfred();
}