use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    command: String,
}

impl Program {
    pub fn new(command: String) -> Program {
        Program { command }
    }

    pub fn command(&self) -> &str {
        &self.command
    }
    pub fn is_valid_command(command: &str) -> bool {
        command.contains("%path%")
    }
}
