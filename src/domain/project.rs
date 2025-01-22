use std::fs::DirEntry;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Project {
    name: String,
    modify: SystemTime,
}

impl Project {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn modify(&self) -> &SystemTime {
        &self.modify
    }
}

impl TryFrom<DirEntry> for Project {
    type Error = String;

    fn try_from(dir_entry: DirEntry) -> Result<Self, Self::Error> {
        let name = dir_entry
            .file_name()
            .to_str()
            .ok_or_else(|| format!("Failed to convert {:?} to Project", dir_entry))?
            .to_string();

        let modify = dir_entry
            .metadata()
            .map_err(|err| err.to_string())?
            .modified()
            .map_err(|err| err.to_string())?;

        Ok(Project { name, modify })
    }
}
