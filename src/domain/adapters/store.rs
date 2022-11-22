use crate::domain::models::Branch;

pub trait Store {
    fn insert_or_update(&self, branch: &Branch) -> anyhow::Result<()>;

    fn get(&self, branch: &str, repo: &str) -> anyhow::Result<Branch>;

    fn close(self) -> anyhow::Result<()>;
}
