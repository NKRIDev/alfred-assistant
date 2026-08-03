pub struct UserCommand {
    pub name: String,
    pub args: Vec<String>,
}

/*
Create UserCommand struct to separate name and args command
 */
pub fn parser(input: &str) -> UserCommand {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    let name = parts.first().unwrap_or(&"").to_string();
    let args = parts[1..].iter().map(|arg| arg.to_string()).collect();

    UserCommand{name, args}
}