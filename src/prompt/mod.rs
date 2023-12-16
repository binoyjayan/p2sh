use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};

pub struct Prompt {
    history: BasicHistory,
}

impl Prompt {
    pub fn new(entries: usize) -> Self {
        Prompt {
            history: BasicHistory::new().max_entries(entries).no_duplicates(true),
        }
    }

    pub fn show(&mut self) -> Result<String, dialoguer::Error> {
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(">> ")
            .history_with(&mut self.history)
            .allow_empty(true)
            .interact_text()
    }
}
