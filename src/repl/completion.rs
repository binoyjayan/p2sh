use dialoguer::{theme::ColorfulTheme, Completion, Select};

pub struct Commands {
    options: Vec<String>,
}

impl Commands {
    pub fn new(options: &[String]) -> Self {
        Commands {
            options: options.to_vec(),
        }
    }
}

impl Completion for Commands {
    /// Simple completion implementation based on substring
    fn get(&self, input: &str) -> Option<String> {
        let matches = self
            .options
            .iter()
            .filter(|option| option.starts_with(input))
            .collect::<Vec<_>>();

        match matches.len() {
            1 => Some(matches[0].to_string()),
            n if n > 1 => {
                // Default to the first item if an error occurs
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("choose from:")
                    .items(&matches[..])
                    .interact_opt()
                    .unwrap_or(None);
                // Return the selected item or None
                selection.map(|idx| matches[idx].to_string())
            }
            _ => None,
        }
    }
}
