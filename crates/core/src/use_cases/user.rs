use std::sync::Arc;

use uuid::Uuid;

use crate::{
    Error, RepositoryError,
    models::{NewUser, User},
    services::{RepositoryService, read_only_transaction, transaction},
};

#[async_trait::async_trait]
pub trait UserUseCases: Send + Sync {
    async fn add_user(&self, user: NewUser) -> Result<User, Error>;
    async fn update_user(&self, user: User) -> Result<User, Error>;
    async fn list_users(&self, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error>;
    async fn delete_user(&self, id: i64) -> Result<User, Error>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
    async fn find_by_token(&self, token: Uuid) -> Result<Option<User>, Error>;
}

pub(crate) struct UserUseCasesImpl {
    repository_service: Arc<RepositoryService>,
}

impl UserUseCasesImpl {
    pub(crate) fn new(repository_service: Arc<RepositoryService>) -> Self {
        Self { repository_service }
    }
}

#[async_trait::async_trait]
impl UserUseCases for UserUseCasesImpl {
    #[tracing::instrument(level = "trace", skip(self, user))]
    async fn add_user(&self, user: NewUser) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();
        let age = user.age;

        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let mut user = user_service.add_user(tx, user).await?;
                user_info_service.add_info(tx, user.token, age).await?;
                user.age = age;
                Ok(user)
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self, user))]
    async fn update_user(&self, user: User) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();
        let age = user.age;

        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let mut updated_user = user_service.update_user(tx, user).await?;
                user_info_service.update_info(tx, updated_user.token, age).await?;
                updated_user.age = age;
                Ok(updated_user)
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn list_users(&self, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();

        read_only_transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let mut users = user_service.list_users(tx, start_id, page_size).await?;
                let tokens: Vec<_> = users.iter().map(|u| u.token).collect();
                let infos = user_info_service.find_by_tokens(tx, &tokens).await?;

                let info_map: std::collections::HashMap<_, _> = infos.into_iter().map(|i| (i.user_token, i.age)).collect();

                for user in &mut users {
                    if let Some(&age) = info_map.get(&user.token) {
                        user.age = age;
                    }
                }
                Ok(users)
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn delete_user(&self, id: i64) -> Result<User, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();

        transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let mut user = user_service
                    .find_by_id(tx, id)
                    .await?
                    .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;

                // Get age before deletion (cascade will delete user_info)
                if let Some(info) = user_info_service.find_by_token(tx, user.token).await? {
                    user.age = info.age;
                }

                user_service.delete_user(tx, user).await
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();

        read_only_transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let Some(mut user) = user_service.find_by_id(tx, id).await? else {
                    return Ok(None);
                };
                if let Some(info) = user_info_service.find_by_token(tx, user.token).await? {
                    user.age = info.age;
                }
                Ok(Some(user))
            })
        })
        .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn find_by_token(&self, token: Uuid) -> Result<Option<User>, Error> {
        let user_service = self.repository_service.user_service.clone();
        let user_info_service = self.repository_service.user_info_service.clone();

        read_only_transaction(&*self.repository_service.repository, |tx| {
            Box::pin(async move {
                let Some(mut user) = user_service.find_by_token(tx, token).await? else {
                    return Ok(None);
                };
                if let Some(info) = user_info_service.find_by_token(tx, user.token).await? {
                    user.age = info.age;
                }
                Ok(Some(user))
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };

    use uuid::Uuid;

    use super::{UserUseCases, UserUseCasesImpl};
    use crate::{
        Error, RepositoryError,
        models::{NewUser, User, user_info::UserInfo},
        services::{Repository, RepositoryService, Transaction, UserInfoService, UserService},
    };

    // ===================
    // Mock Transaction
    // ===================
    struct MockTransaction;

    #[async_trait::async_trait]
    impl Transaction for MockTransaction {
        fn as_any(&self) -> &dyn Any {
            self
        }

        async fn commit(self: Box<Self>) -> Result<(), Error> {
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> Result<(), Error> {
            Ok(())
        }
    }

    // ===================
    // Mock Repository
    // ===================
    struct MockRepository;

    #[async_trait::async_trait]
    impl Repository for MockRepository {
        async fn begin(&self) -> Result<Box<dyn Transaction>, Error> {
            Ok(Box::new(MockTransaction))
        }

        async fn begin_read_only(&self) -> Result<Box<dyn Transaction>, Error> {
            Ok(Box::new(MockTransaction))
        }

        async fn close(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    // ===================
    // Mock UserService
    // ===================
    #[derive(Default)]
    struct MockUserService {
        add_user_result: Mutex<Option<Result<User, Error>>>,
        update_user_result: Mutex<Option<Result<User, Error>>>,
        delete_user_result: Mutex<Option<Result<User, Error>>>,
        find_by_id_result: Mutex<Option<Result<Option<User>, Error>>>,
        find_by_token_result: Mutex<Option<Result<Option<User>, Error>>>,
        list_users_result: Mutex<Option<Result<Vec<User>, Error>>>,
    }

    impl MockUserService {
        fn with_add_user_result(self, result: Result<User, Error>) -> Self {
            *self.add_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_update_user_result(self, result: Result<User, Error>) -> Self {
            *self.update_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_delete_user_result(self, result: Result<User, Error>) -> Self {
            *self.delete_user_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_id_result(self, result: Result<Option<User>, Error>) -> Self {
            *self.find_by_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_list_users_result(self, result: Result<Vec<User>, Error>) -> Self {
            *self.list_users_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_token_result(self, result: Result<Option<User>, Error>) -> Self {
            *self.find_by_token_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait::async_trait]
    impl UserService for MockUserService {
        async fn add_user(&self, _tx: &dyn Transaction, _user: NewUser) -> Result<User, Error> {
            self.add_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn update_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            self.update_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn delete_user(&self, _tx: &dyn Transaction, _user: User) -> Result<User, Error> {
            self.delete_user_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn list_users(&self, _tx: &dyn Transaction, _start_id: Option<i64>, _page_size: Option<u64>) -> Result<Vec<User>, Error> {
            self.list_users_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn find_by_id(&self, _tx: &dyn Transaction, _id: i64) -> Result<Option<User>, Error> {
            self.find_by_id_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }

        async fn find_by_email(&self, _tx: &dyn Transaction, _email: &str) -> Result<Option<User>, Error> {
            Err(Error::Message("Not implemented in mock".into()))
        }

        async fn find_by_token(&self, _tx: &dyn Transaction, _token: Uuid) -> Result<Option<User>, Error> {
            self.find_by_token_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Err(Error::Message("No mock result configured".into())))
        }
    }

    // ===================
    // Mock UserInfoService
    // ===================
    #[derive(Default)]
    struct MockUserInfoService {
        add_info_result: Mutex<Option<Result<UserInfo, Error>>>,
        update_info_result: Mutex<Option<Result<UserInfo, Error>>>,
        find_by_token_result: Mutex<Option<Result<Option<UserInfo>, Error>>>,
        find_by_tokens_result: Mutex<Option<Result<Vec<UserInfo>, Error>>>,
    }

    impl MockUserInfoService {
        fn with_add_info_result(self, result: Result<UserInfo, Error>) -> Self {
            *self.add_info_result.lock().unwrap() = Some(result);
            self
        }

        fn with_update_info_result(self, result: Result<UserInfo, Error>) -> Self {
            *self.update_info_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_token_result(self, result: Result<Option<UserInfo>, Error>) -> Self {
            *self.find_by_token_result.lock().unwrap() = Some(result);
            self
        }

        fn with_find_by_tokens_result(self, result: Result<Vec<UserInfo>, Error>) -> Self {
            *self.find_by_tokens_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait::async_trait]
    impl UserInfoService for MockUserInfoService {
        async fn add_info(&self, _tx: &dyn Transaction, _user_token: Uuid, _age: i16) -> Result<UserInfo, Error> {
            self.add_info_result.lock().unwrap().take().unwrap_or_else(|| Ok(UserInfo::default()))
        }

        async fn update_info(&self, _tx: &dyn Transaction, _user_token: Uuid, _age: i16) -> Result<UserInfo, Error> {
            self.update_info_result.lock().unwrap().take().unwrap_or_else(|| Ok(UserInfo::default()))
        }

        async fn find_by_token(&self, _tx: &dyn Transaction, _user_token: Uuid) -> Result<Option<UserInfo>, Error> {
            self.find_by_token_result.lock().unwrap().take().unwrap_or_else(|| Ok(None))
        }

        async fn find_by_tokens(&self, _tx: &dyn Transaction, _user_tokens: &[Uuid]) -> Result<Vec<UserInfo>, Error> {
            self.find_by_tokens_result.lock().unwrap().take().unwrap_or_else(|| Ok(Vec::new()))
        }
    }

    // ===================
    // Test Helpers
    // ===================
    fn create_use_cases(mock_user_service: MockUserService) -> UserUseCasesImpl {
        create_use_cases_with_info(mock_user_service, MockUserInfoService::default())
    }

    fn create_use_cases_with_info(mock_user_service: MockUserService, mock_user_info_service: MockUserInfoService) -> UserUseCasesImpl {
        let repository_service = Arc::new(RepositoryService {
            repository: Arc::new(MockRepository),
            user_service: Arc::new(mock_user_service),
            user_info_service: Arc::new(mock_user_info_service),
        });
        UserUseCasesImpl::new(repository_service)
    }

    // ===================
    // Tests: add_user
    // ===================
    #[tokio::test]
    async fn test_add_user_success() {
        let expected_user = User::test(1, "John Doe", "john@example.com");
        let mock_user_service = MockUserService::default().with_add_user_result(Ok(expected_user.clone()));
        let user_info = UserInfo {
            user_token: expected_user.token,
            age: 30,
            ..Default::default()
        };
        let mock_user_info_service = MockUserInfoService::default().with_add_info_result(Ok(user_info));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let new_user = NewUser {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        };

        let result = use_cases.add_user(new_user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.age, 30);
    }

    #[tokio::test]
    async fn test_add_user_propagates_error() {
        let mock_service = MockUserService::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate email".into()))));
        let use_cases = create_use_cases(mock_service);

        let new_user = NewUser {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        };

        let result = use_cases.add_user(new_user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::Constraint(_))));
    }

    // ===================
    // Tests: update_user
    // ===================
    #[tokio::test]
    async fn test_update_user_success() {
        let updated_user = User::test(1, "John Updated", "john.updated@example.com");
        let mock_user_service = MockUserService::default().with_update_user_result(Ok(updated_user.clone()));
        let user_info = UserInfo {
            user_token: updated_user.token,
            age: 35,
            ..Default::default()
        };
        let mock_user_info_service = MockUserInfoService::default().with_update_info_result(Ok(user_info));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let mut user = User::test(1, "John Doe", "john@example.com");
        user.age = 35;

        let result = use_cases.update_user(user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "John Updated");
        assert_eq!(user.email, "john.updated@example.com");
        assert_eq!(user.age, 35);
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mock_service = MockUserService::default().with_update_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let use_cases = create_use_cases(mock_service);

        let user = User::test(999, "Nonexistent", "none@example.com");

        let result = use_cases.update_user(user).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }

    // ===================
    // Tests: find_by_id
    // ===================
    #[tokio::test]
    async fn test_find_by_id_found() {
        let expected_user = User::test(1, "John Doe", "john@example.com");
        let mock_user_service = MockUserService::default().with_find_by_id_result(Ok(Some(expected_user.clone())));
        let user_info = UserInfo {
            user_token: expected_user.token,
            age: 30,
            ..Default::default()
        };
        let mock_user_info_service = MockUserInfoService::default().with_find_by_token_result(Ok(Some(user_info)));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let result = use_cases.find_by_id(1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.age, 30);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mock_service = MockUserService::default().with_find_by_id_result(Ok(None));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.find_by_id(999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ===================
    // Tests: list_users
    // ===================
    #[tokio::test]
    async fn test_list_users_success() {
        let user1 = User::test(1, "John Doe", "john@example.com");
        let user2 = User::test(2, "Jane Doe", "jane@example.com");
        let users = vec![user1.clone(), user2.clone()];
        let mock_user_service = MockUserService::default().with_list_users_result(Ok(users));
        let user_infos = vec![
            UserInfo {
                user_token: user1.token,
                age: 30,
                ..Default::default()
            },
            UserInfo {
                user_token: user2.token,
                age: 25,
                ..Default::default()
            },
        ];
        let mock_user_info_service = MockUserInfoService::default().with_find_by_tokens_result(Ok(user_infos));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let result = use_cases.list_users(None, None).await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "John Doe");
        assert_eq!(users[0].age, 30);
        assert_eq!(users[1].name, "Jane Doe");
        assert_eq!(users[1].age, 25);
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let mock_service = MockUserService::default().with_list_users_result(Ok(vec![]));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.list_users(None, None).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ===================
    // Tests: delete_user
    // ===================
    #[tokio::test]
    async fn test_delete_user_success() {
        let user_to_find = User::test(1, "John Doe", "john@example.com");
        // The mock delete_user returns a user with age=30 since the use case sets
        // the age before calling delete_user
        let user_to_delete = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock_user_service = MockUserService::default()
            .with_find_by_id_result(Ok(Some(user_to_find.clone())))
            .with_delete_user_result(Ok(user_to_delete));
        let user_info = UserInfo {
            user_token: user_to_find.token,
            age: 30,
            ..Default::default()
        };
        let mock_user_info_service = MockUserInfoService::default().with_find_by_token_result(Ok(Some(user_info)));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let result = use_cases.delete_user(1).await;

        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert_eq!(deleted.id, 1);
        assert_eq!(deleted.name, "John Doe");
        assert_eq!(deleted.age, 30);
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mock_service = MockUserService::default().with_find_by_id_result(Ok(None));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.delete_user(999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RepositoryError(RepositoryError::NotFound)));
    }

    // ===================
    // Tests: find_by_token
    // ===================
    #[tokio::test]
    async fn test_find_by_token_found() {
        let expected_user = User::test(1, "John Doe", "john@example.com");
        let mock_user_service = MockUserService::default().with_find_by_token_result(Ok(Some(expected_user.clone())));
        let user_info = UserInfo {
            user_token: expected_user.token,
            age: 30,
            ..Default::default()
        };
        let mock_user_info_service = MockUserInfoService::default().with_find_by_token_result(Ok(Some(user_info)));
        let use_cases = create_use_cases_with_info(mock_user_service, mock_user_info_service);

        let result = use_cases.find_by_token(expected_user.token).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.age, 30);
    }

    #[tokio::test]
    async fn test_find_by_token_not_found() {
        let mock_service = MockUserService::default().with_find_by_token_result(Ok(None));
        let use_cases = create_use_cases(mock_service);

        let result = use_cases.find_by_token(Uuid::new_v4()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
