mod fakers;

use std::collections::HashMap;

use fake::{Fake, Faker};
use inkan::{
    domain::{
        adapters::{CheckoutStatus, Store},
        commands::checkout::{handler, Checkout},
        errors::{GitError, PersistError},
        models::Branch,
    },
    template_config::{BranchConfig, CommitConfig, TemplateConfig},
};

use crate::fakers::{fake_config, fake_context, GitCommandMock};

#[test]
fn checkout_success_with_ticket() -> anyhow::Result<()> {
    // Arrange
    let repo = Faker.fake::<String>();

    let command = Checkout {
        ticket: Some(Faker.fake()),
        ..fake_checkout_args()
    };

    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(command.name.clone()),
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config())?;

    // Act
    handler(
        &context.git,
        &context.store,
        fake_template_config(),
        command.clone(),
    )?;

    // Assert
    let branch = context.store.get_branch(&command.name, &repo)?;
    let name = format!(
        "{}-{}",
        &git_commands.repo.unwrap(),
        &git_commands.branch_name.unwrap()
    );

    let expected = Branch {
        name,
        ticket: command.ticket.unwrap(),
        ..branch.clone()
    };

    assert_eq!(branch, expected);

    context.close()?;

    Ok(())
}

#[test]
fn checkout_uses_branch_template() {
    let repo = Faker.fake::<String>();

    let command = Checkout {
        ticket: Some(Faker.fake()),
        scope: Some(Faker.fake()),
        ..fake_checkout_args()
    };

    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(command.name.clone()),
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config()).unwrap();

    let template = TemplateConfig {
        branch: Some(BranchConfig {
            content: "{branch_name}-{scope}-{ticket_num}".into(),
        }),
        ..fake_template_config()
    };

    // Act
    handler(&context.git, &context.store, template, command.clone()).unwrap();

    let branch_name = format!(
        "{}-{}-{}",
        &command.name,
        &command.scope.clone().unwrap(),
        &command.ticket.clone().unwrap()
    );

    // Assert
    let branch = context.store.get_branch(&branch_name, &repo).unwrap();

    let name = format!("{}-{}", &git_commands.repo.unwrap(), branch_name);
    let expected: Branch = Branch {
        name,
        ticket: command.ticket.unwrap(),
        link: command.link,
        scope: command.scope,
        created: branch.created,
        data: None,
    };

    assert_eq!(branch, expected);
    context.close().unwrap();
}

#[test]
fn checkout_with_branch_already_exists_does_not_error() -> anyhow::Result<()> {
    // Arrange
    let repo = Faker.fake::<String>();

    let command = fake_checkout_args();

    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(command.name.clone()),
        checkout_res: |_, status| {
            assert_eq!(status, CheckoutStatus::New);
            Ok(())
        },
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config())?;

    // Act
    handler(
        &context.git,
        &context.store,
        fake_template_config(),
        command.clone(),
    )?;

    // Assert
    let branch = context.store.get_branch(&command.name, &repo)?;
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
        created: branch.created,
        data: None,
    };

    assert_eq!(branch, expected);

    context.close()?;

    Ok(())
}

#[test]
fn checkout_on_fail_to_checkout_branch_nothing_is_persisted() {
    // Arrange
    let command = fake_checkout_args();

    let repo = Faker.fake::<String>();
    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(command.name.clone()),
        checkout_res: |_, _| {
            Err(GitError::Validation {
                message: "failed to create or checkout existing branch!".into(),
            })
        },
        commit_res: |_, _| panic!("commit method should not be called"),
        template_file_path: || panic!("template should not be needed"),
    };

    let context = fake_context(git_commands.clone(), fake_config()).unwrap();

    // Act
    handler(
        &context.git,
        &context.store,
        fake_template_config(),
        command.clone(),
    )
    .unwrap_err();

    // Assert
    let error = context
        .store
        .get_branch(&command.name, &repo)
        .expect_err("Expected error as there should be no stored branches.");

    assert!(matches!(error, PersistError::NotFound { ref name } if name == "branch"));
    assert_eq!(
        error.to_string(),
        "Requested \"branch\" not found in persisted store"
    );
    context.close().unwrap();
}

#[test]
fn checkout_success_without_ticket_uses_branch_name() -> anyhow::Result<()> {
    // Arrange
    let repo = Faker.fake::<String>();

    let command = Checkout {
        ticket: None,
        ..fake_checkout_args()
    };

    let git_commands = GitCommandMock {
        repo: Ok(repo.clone()),
        branch_name: Ok(command.name.clone()),
        ..GitCommandMock::fake()
    };

    let context = fake_context(git_commands.clone(), fake_config())?;

    // Act
    handler(
        &context.git,
        &context.store,
        fake_template_config(),
        command.clone(),
    )?;

    // Assert
    let branch = context.store.get_branch(&command.name, &repo)?;
    let name = format!(
        "{}-{}",
        &git_commands.repo.unwrap(),
        &git_commands.branch_name.unwrap()
    );

    let expected = Branch {
        name,
        ticket: command.name,
        scope: command.scope,
        link: command.link,
        data: None,
        created: branch.created,
    };

    assert_eq!(branch, expected);

    context.close()?;

    Ok(())
}

pub fn fake_checkout_args() -> Checkout {
    Checkout {
        name: Faker.fake(),
        ticket: Some(Faker.fake()),
        link: Some(Faker.fake()),
        scope: Some(Faker.fake()),
    }
}

pub fn fake_template_config() -> TemplateConfig {
    TemplateConfig {
        version: 1,
        branch: None,
        commit: CommitConfig {
            templates: HashMap::new(),
        },
    }
}
