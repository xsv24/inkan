use crate::branch::{get_branch_name, Branch};
use clap::Args;
use rusqlite::Connection;

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    #[clap(short, long, value_parser)]
    pub message: Option<String>,
}

impl Arguments {
    pub fn commit_message(&self, template: String, conn: &Connection) -> anyhow::Result<String> {
        let ticket_num = match &self.ticket {
            Some(num) => num.into(),
            None => Branch::get(&get_branch_name()?, &conn)?.ticket,
        };

        let contents = template.replace("{ticket_num}", &format!("[{}]", ticket_num));

        let contents = match &self.message {
            Some(message) => contents.replace("{message}", &message),
            None => contents.replace("{message}", ""),
        };

        Ok(contents)
    }
}
