use tokio::io::{self, AsyncBufReadExt, BufReader};
use crate::core::alfred::Alfred;
use crate::core::dream::DreamTimer;
use crate::core::orchestrator::Orchestrator;
/*
Returns the value the user enters in the console
 */
async fn input_command(reader: &mut BufReader<io::Stdin>) -> String {
    let mut input = String::new();
    reader.read_line(&mut input).await.expect("Failed to read input.");
    input.trim().to_string()
}

/*
Main loop
 */
pub async fn start_alfred(mut alfred: Alfred) {
    println!("Welcome to Alfred Assistant");

    /*
    Tokio lib
     */
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    /*
    Alfred loop
     */
    loop {
        println!("> ");
        let input = input_command(&mut reader).await;
        //  let parser = parser(&input);
        let reply = Orchestrator::ask_alfred(&input, &mut alfred).await;
        println!("{}", reply);

        //Check if input is "quit", break loop
        if input == "quit" {
            DreamTimer::trigger(alfred.database_url).await;
            break;
        }
    }
}