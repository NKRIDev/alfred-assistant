use std::io;
use crate::commands::parser::parser;
use crate::commands::registry::init_commands;
use crate::services::application::ApplicationService;
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
pub fn start_alfred(app_service: ApplicationService) {
    println!("Welcome to Alfred Assistant");

    //Init registry command system
    let registry = init_commands(app_service);

    /*
    Alfred loop
     */
    loop {
        println!("> ");
        let input = input_command();
        let parser = parser(&input);
        let reply = registry.execute(&parser.name, &parser.args);
        println!("{}", reply);

        //Check if input is "quit", break loop
        if input == "quit" {
            break;
        }
    }
}