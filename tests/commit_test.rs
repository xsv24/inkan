mod fakers;

use crate::fakers::{fake_config, fake_context, GitCommandMock};
use fake::{Fake, Faker};
use git_kit::{
    adapters::sqlite::Sqlite,
    domain::{
        adapters::{CommitMsgStatus, Git, Store},
        commands::commit::{handler, Commit},
        models::Branch,
    },
    template_config::Template,
};

#[test]
fn commit_message_with_all_arguments_are_injected_into_the_template_with_nothing_persisted(
) -> anyhow::Result<()> {
    // Arrange
    let template_config = fake_template();

    let git_mock = GitCommandMock {
        commit_res: |_, complete| {
            assert_eq!(CommitMsgStatus::Completed, complete);
            Ok(())
        },
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_mock, fake_config())?;

    let args = Commit {
        ticket: Some(Faker.fake()),
        message: Some(Faker.fake()),
        scope: Some(Faker.fake()),
        template: template_config,
        ..fake_commit_args()
    };

    // Act
    let contents = handler(&context.git, &context.store, args.clone())
        .expect("Error performing 'commit' action");

    // Assert
    let expected = format!(
        "[{}] message: '{}', scope: '{}', link: '{}'",
        args.ticket.clone().unwrap(),
        args.message.clone().unwrap(),
        args.scope.unwrap(),
        ""
    );
    assert_eq!(expected, contents);

    context.close()?;

    Ok(())
}

#[test]
fn commit_message_with_no_args_or_stored_branch_defaults_correctly() -> anyhow::Result<()> {
    // Arrange
    let template_config = fake_template();

    let git_mock = GitCommandMock {
        commit_res: |_, complete| {
            assert_eq!(CommitMsgStatus::InComplete, complete);
            Ok(())
        },
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_mock, fake_config())?;

    let args = Commit {
        ticket: None,
        message: None,
        scope: None,
        template: template_config,
    };

    // Act
    let contents = handler(&context.git, &context.store, args.clone())
        .expect("Error performing 'commit' action");

    // Assert
    assert_eq!("message: '', scope: '', link: ''", contents);

    context.close()?;

    Ok(())
}

#[test]
fn commit_message_with_no_commit_args_defaults_to_stored_branch_values() -> anyhow::Result<()> {
    // Arrange
    let template_config = fake_template();

    let args = Commit {
        template: template_config,
        message: Some(Faker.fake()),
        ticket: None,
        scope: None,
    };

    let context = fake_context(GitCommandMock::fake(), fake_config())?;

    let branch_name = Some(context.git.branch_name()?);
    let repo_name = Some(context.git.repository_name()?);
    let ticket = None;
    let branch = Branch {
        link: Some(Faker.fake()),
        scope: Some(Faker.fake()),
        ..fake_branch(branch_name.clone(), repo_name, ticket)?
    };

    setup_db(&context.store, Some(&branch))?;

    // Act
    let commit_message = handler(&context.git, &context.store, args.clone())
        .expect("Error performing 'commit' action");

    // Assert
    let expected = format!(
        "[{}] message: '{}', scope: '{}', link: '{}'",
        branch_name.unwrap(),
        args.message.unwrap(),
        branch.scope.unwrap(),
        branch.link.unwrap()
    );

    assert_eq!(expected.trim(), commit_message);

    context.close()?;

    Ok(())
}

fn setup_db(store: &Sqlite, branch: Option<&Branch>) -> anyhow::Result<()> {
    if let Some(branch) = branch {
        store.persist_branch(branch.into())?;
    }

    Ok(())
}

fn fake_commit_args() -> Commit {
    let template = fake_template();

    Commit {
        template,
        ticket: Faker.fake(),
        message: Faker.fake(),
        scope: Faker.fake(),
    }
}

fn fake_template() -> Template {
    Template {
        description: Faker.fake(),
        content: "[{ticket_num}] message: '{message}', scope: '{scope}', link: '{link}'".into(),
    }
}

fn fake_branch(
    name: Option<String>,
    repo: Option<String>,
    ticket: Option<String>,
) -> anyhow::Result<Branch> {
    let name = name.unwrap_or(Faker.fake());
    let repo = repo.unwrap_or(Faker.fake());

    Ok(Branch::new(
        &name,
        &repo,
        ticket,
        Faker.fake(),
        Faker.fake(),
    ))
}
