use std::sync::Arc;

use crate::{
    Error, RepositoryError,
    models::NewUserBuilder,
    services::{RepositoryService, read_only_transaction, transaction},
};

pub(crate) async fn sample_database_work(repository_service: &Arc<RepositoryService>) -> Result<(), Error> {
    let tx = repository_service.repository.begin().await?;
    let user = NewUserBuilder::default()
        .name("Fred Wombat".into())
        .email("fred@wombat.com".into())
        .build()
        .unwrap();

    let existing_user = repository_service.user_service.find_by_email(&*tx, &user.email).await?;
    let user = if let Some(user) = existing_user {
        tracing::info!("Found user");
        let mut user = user;
        user.email = "also_fred@wombat.com".into();
        let user = repository_service.user_service.update_user(&*tx, user).await?;
        dbg!(&user);
        user
    } else {
        tracing::info!("Not found");
        repository_service.user_service.add_user(&*tx, user).await?
    };
    dbg!(&user);

    tx.commit().await?;

    let user_service = repository_service.user_service.clone();
    let mary = transaction(&*repository_service.repository, |tx| {
        Box::pin(async move {
            let user = NewUserBuilder::default()
                .name("Mary Wombat".into())
                .email("mary@wombat.com".into())
                .build()
                .unwrap();

            let existing_user = user_service.find_by_email(tx, &user.email).await?;
            let mary = if let Some(mary) = existing_user {
                tracing::info!("Mary already exists in the database");
                mary
            } else {
                user_service.add_user(tx, user).await?
            };
            Ok(mary)
        })
    })
    .await?;
    dbg!(&mary);

    let user_service = repository_service.user_service.clone();
    let bill = read_only_transaction(&*repository_service.repository, |tx| {
        Box::pin(async move {
            let user = NewUserBuilder::default()
                .name("Bill Wombat".into())
                .email("bill@wombat.com".into())
                .build()
                .unwrap();

            let existing_user = user_service.find_by_email(tx, &user.email).await?;
            let bill = if let Some(bill) = existing_user {
                tracing::info!("Bill already exists in the database");
                bill
            } else {
                user_service.add_user(tx, user).await?
            };
            Ok(bill)
        })
    })
    .await;
    match bill {
        Err(Error::RepositoryError(RepositoryError::ReadOnly)) => {
            tracing::info!("Got expected ReadOnly error")
        }
        _ => {
            dbg!(&bill);
        }
    }

    Ok(())
}
