use async_trait::async_trait;

/*
An incident detected by a watcher, to be forwarded to Alfred
for assessment and potential action
 */
pub struct WatcherEvent{
    pub source: String,
    pub context: String, //Text provided to the LLM
}

#[async_trait]
pub trait Watcher: Send + Sync {

    /*
    Watcher name
     */
    fn name(&self) -> String;

    /*
    Checked method
     */
    async fn check(&self) -> Result<Vec<WatcherEvent>, String>;

    /*
    Custom prompt
     */
    fn prompt(&self) -> String;
}