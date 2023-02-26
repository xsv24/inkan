mod fakers;

use fake::{Fake, Faker};
use git_kit::domain::{
    adapters::Store,
    commands::context::{handler, Context},
    models::Branch,
};

use crate::fakers::{fake_branch, fake_config, fake_context, GitCommandMock};

#[test]
fn current_success() -> anyhow::Result<()> {
    // Arrange
    let branch_name = Faker.fake::<String>();
    let repo = Faker.fake::<String>();
    let command = Context {
        ticket: Some(Faker.fake()),
        ..fake_context_args()
    };

    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(branch_name.clone()),
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config())?;

    // Act
    handler(&context.git, &context.store, command.clone())?;

    // Assert
    let branch = context.store.get_branch(&branch_name, &repo)?;
    let name = format!(
        "{}-{}",
        &git_commands.repo.unwrap(),
        &git_commands.branch_name.unwrap()
    );

    let expected = Branch {
        name,
        ticket: command.ticket.unwrap(),
        link: command.link,
        scope: command.scope,
        ..branch.clone()
    };
    assert_eq!(branch, expected);

    context.close()?;

    Ok(())
}

#[test]
fn context_with_optionals_none_does_not_overwrite_db() -> anyhow::Result<()> {
    // Arrange
    let branch = Branch {
        data: Some(Faker.fake()),
        link: Some(Faker.fake()),
        scope: Some(Faker.fake()),
        ..fake_branch()
    };

    let repo = Faker.fake::<String>();
    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(branch.name.clone()),
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config())?;

    context.store.persist_branch(&branch)?;

    // Act
    let command = Context {
        ticket: None,
        scope: None,
        link: None,
    };

    handler(&context.git, &context.store, command.clone())?;

    // Assert
    let updated_branch = context.store.get_branch(&branch.name, &repo)?;
    let name = format!(
        "{}-{}",
        &git_commands.repo.unwrap(),
        &git_commands.branch_name.unwrap()
    );

    let expected = Branch {
        name,
        ..updated_branch.clone()
    };
    assert_eq!(updated_branch, expected);
    Ok(())
}

fn fake_context_args() -> Context {
    Context {
        ticket: Faker.fake(),
        scope: Faker.fake(),
        link: Faker.fake(),
    }
}
