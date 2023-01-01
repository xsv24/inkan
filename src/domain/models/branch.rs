use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub ticket: String,
    pub created: DateTime<Utc>,
    pub data: Option<Vec<u8>>,
    pub link: Option<String>,
    pub scope: Option<String>,
}

impl Branch {
    pub fn new(
        name: &str,
        repo: &str,
        ticket: Option<String>,
        link: Option<String>,
        scope: Option<String>,
    ) -> anyhow::Result<Branch> {
        Ok(Branch {
            name: format!("{}-{}", repo.trim(), name.trim()),
            created: Utc::now(),
            ticket: ticket.unwrap_or_else(|| name.into()),
            link,
            scope,
            data: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn creating_branch_with_ticket_populates_correctly() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let repo = Faker.fake::<String>();
        let name = Faker.fake::<String>();
        let ticket = Faker.fake::<String>();
        let scope = Faker.fake::<String>();
        let link = Faker.fake::<String>();

        // Act
        let branch = Branch::new(
            &name,
            &repo,
            Some(ticket.clone()),
            Some(link.clone()),
            Some(scope.clone()),
        )?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo, &name));
        assert_eq!(branch.ticket, ticket);
        assert_eq!(branch.scope.unwrap(), scope);
        assert_eq!(branch.link.unwrap(), link);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }

    #[test]
    fn creating_branch_without_ticket_defaults_to_name() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();

        // Act
        let branch = Branch::new(&name, &repo, None, None, None)?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo, &name));
        assert_eq!(branch.ticket, name);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }

    #[test]
    fn branch_name_is_trimmed() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let name = format!("{}\n", Faker.fake::<String>());
        let ticket = Faker.fake::<String>();
        let repo = Faker.fake::<String>();

        // Act
        let branch = Branch::new(&name, &repo, Some(ticket.clone()), None, None)?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo.trim(), &name.trim()));
        assert_eq!(branch.ticket, ticket);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }
}
