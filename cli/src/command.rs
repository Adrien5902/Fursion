use crate::error::Error;

#[derive(Debug)]
pub struct Command {
    pub function: for<'a> fn(args: &'a [String]) -> Result<&'a str, Error>,
    pub name: &'static str,
    pub alias: Option<&'static str>,
}

pub const COMMANDS: &[Command] = &[Command {
    name: "commit",
    alias: None,
    function: |arg| Ok("coucou"),
}];

impl Command {
    pub fn from_str<'a>(value: &'a str) -> Result<&Self, Error> {
        COMMANDS
            .iter()
            .find(|c| c.name == value || c.alias == Some(value))
            .ok_or(Error::CommandNotFound(value.to_string()))
    }
}
