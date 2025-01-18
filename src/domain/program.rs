use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    name: String,
    command: String,
}

impl Program {
    pub fn new(name: String, command: String) -> Program {
        Program { name, command }
    }

    pub fn command(&self) -> &str {
        &self.command
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_valid_command(command: &str) -> bool {
        command.contains("%path%")
    }
}
