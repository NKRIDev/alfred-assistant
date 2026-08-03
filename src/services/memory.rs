use chrono::Utc;
use rusqlite::{params, Connection, Result};
use crate::services::ollama::OllamaService;
use std::fs;

/*
Connection link between the SQLite
database and the program
 */
pub struct MemoryService{
    connection: Connection,
}

pub struct MemoryEvent {
    id: i64,
    role: String,
    content: String,
    tool_name: Option<String>,
    created_at: String
}

impl MemoryEvent {

    /*
    Displays an event in the expected string format
     */
    fn to_string(&self) -> String {
        format!("[{}] {} : {}", self.created_at, self.role, self.content)
    }
}

impl MemoryService {

    /*
    Created struct and init connection with
    database
     */
    pub fn new(database_path: String) -> Result<Self> {
        let connection = Connection::open(database_path)?;

        /*
        Create table if not exist
         */
        connection.execute("\
            CREATE TABLE IF NOT EXISTS events (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                role TEXT NOT NULL, \
                content TEXT NOT NULL, \
                tool_name TEXT,\
                created_at TIMESTAMP NOT NULL)\
        ", [],)?;

        Ok(Self {connection})
    }

    /*
    Adds an event to the database
     */
    pub fn log_event(&self, role: &str, content: &str, tool_name: Option<&str>) -> Result<()> {
        let query = "INSERT INTO events (role, content, tool_name, created_at) VALUES (?1, ?2, ?3, ?4)";
        self.connection.execute(query, params![role, content, tool_name, Utc::now().to_rfc3339()])?;
        Ok(())
    }

    /*
    Retrieve all wind data from the database.
     */
    pub fn get_role_content(&self) -> Result<Vec<(String, String)>> {
        let query = "SELECT role, content FROM events";
        let mut prepared_statement = self.connection.prepare(query)?;
        let results = prepared_statement.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        Ok(results.filter_map(|res| res.ok()).collect())
    }

    /*
    Recover all events from database
     */
    pub fn get_events(&self) -> Result<Vec<MemoryEvent>> {
        let query = "SELECT id, role, content, tool_name, created_at FROM events";
        let mut prepared_statement = self.connection.prepare(query)?;
        let results = prepared_statement.query_map([], |row| {
            Ok(MemoryEvent{
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                tool_name: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /*
    Returns all events and creates a "memory.md" readme
    file that stores the data.

    TODO : In the future, why not create target Markdown files
     for specific domains (user, project, preferences, etc.) that are
     loaded at the right time?
     */
    pub fn dream(&self) -> String {
        let events = match self.get_events() {
            Ok(events) => events,
            Err(error) => return error.to_string(),
        };

        /*
        Check if events is empty
         */
        if events.is_empty() {
            return String::from("No events found");
        }

        /*
        Build events list
         */
        let logs = events.iter()
            .filter(|event| event.role != "tool")
            .map(|event| event.to_string())
            .collect::<Vec<_>>().join(" ");

        /*
        Create prompt
         */
        let prompt = format!(
            "You are a memory consolidation system for an AI assistant named Alfred, \
            modeled after human long-term memory systems (Tulving, Squire): semantic, episodic, and procedural memory.\n\n\
            IMPORTANT CONTEXT: Alfred already has a fixed base personality and behavior rules defined elsewhere \
            (butler tone, formal address, general interaction style). Do NOT restate or summarize that base behavior. \
            Only capture information that is a SPECIFIC ADJUSTMENT or EXCEPTION this particular user has expressed, \
            which overrides or refines the default behavior.\n\n\
            Here is the raw log of recent interactions with the user:\n\n{}\n\n\
            Extract information into these categories:\n\n\
            - SEMANTIC MEMORY (facts): stable, context-independent facts about who the user is \
            (name, location, occupation, general personal context)\n\
            - PROCEDURAL MEMORY (user-specific adjustments): ONLY explicit deviations from Alfred's default behavior \
            that this specific user requested (e.g. 'wants to be called by first name instead of Monsieur', \
            'prefers shorter answers than Alfred's default style'). Do NOT include Alfred's standard butler behavior — \
            only what THIS user specifically asked to be different.\n\
            - EPISODIC MEMORY (events): specific decisions or events tied to a moment, with enough context \
            to understand why they matter — do not repeat facts already captured in semantic memory\n\
            - ACTIVE GOALS: what the user is currently working on or has not yet resolved — distinct from \
            long-term memory since it is expected to change or be completed\n\n\
            STRICTLY EXCLUDE any information that is time-sensitive, tied to a specific moment, \
            or that reflects a snapshot of the world rather than a lasting fact about the user \
            (for example: real-time data, measurements, current status of something, one-off factual \
            lookups with no lasting relevance to the user). \
            If in doubt whether a piece of information will still be true or relevant in a week, exclude it. \
            \
            Ignore greetings, small talk, and filler exchanges. \
            Do not invent or infer anything not explicitly stated in the log. \
            \
            CRITICAL FORMATTING RULE: if a category has no genuine content, DO NOT include that section \
            header at all — skip it entirely. Only output headers that have real content underneath. \
            \
            If no durable information is found at all, respond with exactly: \
            'Aucune information marquante pour le moment.' \
            \
            Write the summary in French, using these Markdown headers only when they have content:\n\n\
            ## Mémoire sémantique — Faits\n\
            ## Mémoire procédurale — Ajustements spécifiques\n\
            ## Mémoire épisodique — Événements\n\
            ## Objectifs actifs\n\n\
            Respond only with the Markdown content, no commentary or explanation.",
            logs
        );

        /*
        Message and call ollama API
         */
        let messages = serde_json::json!([
            {"role": "user", "content": prompt},
        ]);

        let response = OllamaService::chat(&messages, &serde_json::json!([]));
        let summary = response["message"]["content"]
            .as_str()
            .unwrap_or("Error: no summary generated")
            .to_string();

        /*
        Write file on memory folder
         */
        if let Err(e) = fs::write("memory/memory.md", &summary) {
            return format!("Error writing memory.md : {}", e);
        }

        format!("memory.md successfully generated ({} events processed", events.len())
    }

    /*
    Returns the assistant's long-term memory
     */
    pub fn load_term_memory() -> String {
        let memory_content = fs::read_to_string("memory/memory.md").unwrap_or_default();
        let edit_file = format!("## Memory of previous conversations \n{}", memory_content);
        edit_file
    }
}