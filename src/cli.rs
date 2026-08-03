use std::io;
use crate::core::alfred::Alfred;
use crate::core::orchestrator::Orchestrator;
/*
Returns the value the user enters in the console
 */
fn input_command() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input.");
    input.trim().to_string()
}

/*
Main loop
 */
pub fn start_alfred(mut alfred: Alfred) {
    println!("Welcome to Alfred Assistant");

    /*
    Alfred loop
     */
    loop {
        println!("> ");
        let input = input_command();
      //  let parser = parser(&input);
        let reply = Orchestrator::ask_alfred(&input, &mut alfred);
        println!("{}", reply);

        //Check if input is "quit", break loop
        if input == "quit" {
            break;
        }
    }
}