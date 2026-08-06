use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;

/*
Connection link between the SQLite
database and the program
 */
pub struct MemoryService{
    connection: Connection,
}

pub struct MemoryEvent {
    role: String,
    content: String,
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
    pub fn new(database_path: &str) -> Result<Self> {
        let connection = Connection::open(database_path)?;

        /*
        Multiple simultaneous readers, one writer + readers and
        if the database is busy, wait 5 seconds before writing
        */
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;

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
    Remove all events
     */
    pub fn clear_events(&self) -> Result<()> {
        self.connection.execute("DELETE FROM events", [])?;
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
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /*
    Get the number of events in db
     */
    pub fn count_events(&self) -> Result<usize, String> {
        let query = "SELECT COUNT(*) FROM events";
        self.connection.query_row(query, [], |row| {
                let count: i64 = row.get(0)?;
                Ok(count as usize)
            }).map_err(|e| e.to_string())
    }

    /*
    Returns all events and creates a "memory.md" readme
    file that stores the data.

    TODO : In the future, why not create target Markdown files
     for specific domains (user, project, preferences, etc.) that are
     loaded at the right time?

     FIX : async execution of the Dream to avoid blocking
     the assistant's response
     */
    pub fn prepared_dream_prompt(&self) -> Result<Option<String>, String> {
        let events = match self.get_events() {
            Ok(events) => events,
            Err(error) => return Ok(None),
        };

        /*
        Check if events is empty
         */
        if events.is_empty() {
            return Ok(None);
        }

        /*
        Load actually long term memory
         */
        let existing_memory = fs::read_to_string("memory/memory.md").unwrap_or_default();

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
            (butler tone, formal address, general interaction style). Do NOT restate or summarize that base behavior.\n\n\
            Here is Alfred's EXISTING long-term memory (may be empty if this is the first consolidation):\n\n{}\n\n\
            Here is the NEW raw log of recent interactions to integrate:\n\n{}\n\n\
            Your task: MERGE the new information into the existing memory, producing a single updated version. \
            - If new information confirms or adds detail to something already in memory, keep the most complete/recent version.\n\
            - If new information CONTRADICTS existing memory (e.g. user moved to a new city), REPLACE the outdated fact \
            with the new one — do not keep both.\n\
            - If new information is genuinely new, add it to the appropriate category.\n\
            - If an ACTIVE GOAL from existing memory appears to be resolved or completed based on the new log, \
            move it out of ACTIVE GOALS (either drop it, or if significant, record it as a past event in episodic memory).\n\
            - Do not duplicate information across categories.\n\n\
            Extract and organize into these categories:\n\n\
            - SEMANTIC MEMORY (facts): stable, context-independent facts about who the user is\n\
            - PROCEDURAL MEMORY (user-specific adjustments): Include something here ONLY if the USER explicitly \
            asked Alfred to change how it behaves (e.g. the user literally said something like \
            'appelle-moi X', 'sois moins formel', 'réponds plus court'). \
            \n\n\
            DO NOT include anything here based on how Alfred itself spoke in the log — Alfred's own tone, \
            word choice, or way of addressing the user is NEVER a source for this category, even if repeated \
            consistently. Only the USER's explicit requests count. \
            \n\n\
            Test before including anything here: can you point to a specific message FROM THE USER (not from Alfred) \
            that explicitly requested this behavior? If not, do not include it. \
            \n\n\
            Example of what NOT to include: 'Alfred addresses the user formally' — this describes Alfred's \
            own behavior, not a user request, so it must be excluded even if true in the log.\n\
            - EPISODIC MEMORY (events): specific decisions or events tied to a moment, with context\n\
            - ACTIVE GOALS: what the user is currently working on or has not yet resolved\n\n\
            STRICTLY EXCLUDE any information that is time-sensitive or reflects a snapshot of the world \
            rather than a lasting fact (real-time data, measurements, one-off lookups). \
            If in doubt whether it will still be true in a week, exclude it. \
            Ignore greetings, small talk, filler exchanges. \
            Do not invent anything not explicitly stated in either the existing memory or the new log. \
            \
            CRITICAL FORMATTING RULE: if a category has no genuine content, omit that section header entirely. \
            \
            If the merged result has no durable information at all, respond with exactly: \
            'Aucune information marquante pour le moment.' \
            \
            Write the result in French, using these Markdown headers only when they have content:\n\n\
            ## Mémoire sémantique — Faits\n\
            ## Mémoire procédurale — Ajustements spécifiques\n\
            ## Mémoire épisodique — Événements\n\
            ## Objectifs actifs\n\n\
            Respond only with the merged Markdown content, no commentary or explanation.",
            existing_memory, logs
        );

        Ok(Some(prompt))
    }


    pub fn finalization_dream(&self, summary: &str) -> Result<String, String> {
        /*
        Write file on memory folder
         */
        if let Err(e) = fs::write("memory/memory.md", summary) {
            return Err(format!("Error writing memory.md: {}", e));
        }
        
        /*
        Delete all events
         */
        self.clear_events().map_err(|e| e.to_string())?;

        Ok(String::from("memory.md successfully generated !"))
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