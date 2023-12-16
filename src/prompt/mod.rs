use dialoguer::{theme::ColorfulTheme, BasicHistory, Completion, Input};

struct Commands {
    options: Vec<String>,
}

impl Commands {
    fn new(options: &[String]) -> Self {
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

        if matches.len() == 1 {
            Some(matches[0].to_string())
        } else {
            None
        }
    }
}

pub struct Prompt {
    history: BasicHistory,
    commands: Commands,
}

impl Prompt {
    pub fn new(entries: usize, commands: &[String]) -> Self {
        Prompt {
            history: BasicHistory::new().max_entries(entries).no_duplicates(true),
            commands: Commands::new(commands),
        }
    }

    pub fn show(&mut self) -> Result<String, dialoguer::Error> {
        let mut input_lines = Vec::new();

        loop {
            let input_line = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(if input_lines.is_empty() { ">>" } else { ">" })
                .history_with(&mut self.history)
                .completion_with(&self.commands)
                .allow_empty(true)
                .interact_text()?;

            // Check if the line ends with a backslash ("\")
            let is_continuation = input_line.trim_end().ends_with('\\');

            // Remove the trailing backslash for multiline input
            let cleaned_line = if is_continuation {
                input_line.trim_end_matches('\\').to_string()
            } else {
                input_line
            };

            input_lines.push(cleaned_line);

            // Break the loop if the line does not end with a backslash
            if !is_continuation {
                break;
            }
        }

        // Join all input lines and return as a single string
        let result = input_lines.join("\n");
        Ok(result)
    }
}
