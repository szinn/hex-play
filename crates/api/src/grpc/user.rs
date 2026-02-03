use std::sync::Arc;

use hex_play_core::services::CoreServices;
use tonic::{Request, Response, Status};

use crate::grpc::{
    error::map_core_error,
    user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest, User as ProtoUser,
        user_service_server::UserService,
    },
};

/// gRPC UserService implementation
pub(crate) struct GrpcUserService {
    core_services: Arc<CoreServices>,
}

impl GrpcUserService {
    pub(crate) fn new(core_services: Arc<CoreServices>) -> Self {
        Self { core_services }
    }
}

#[tonic::async_trait]
impl UserService for GrpcUserService {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn create(&self, request: Request<CreateUserRequest>) -> Result<Response<ProtoUser>, Status> {
        let response = handler::create(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get(&self, request: Request<GetUserRequest>) -> Result<Response<ProtoUser>, Status> {
        let response = handler::get(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_by_token(&self, request: Request<GetUserByTokenRequest>) -> Result<Response<ProtoUser>, Status> {
        let response = handler::get_by_token(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn update(&self, request: Request<UpdateUserRequest>) -> Result<Response<ProtoUser>, Status> {
        let response = handler::update(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn delete(&self, request: Request<DeleteUserRequest>) -> Result<Response<ProtoUser>, Status> {
        let response = handler::delete(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn list(&self, request: Request<ListUsersRequest>) -> Result<Response<ListUsersResponse>, Status> {
        let response = handler::list(&self.core_services, request.into_inner()).await.map_err(map_core_error)?;
        Ok(Response::new(response))
    }
}

/// Server-side handlers (business logic)
pub(crate) mod handler {
    use hex_play_core::{
        Error, RepositoryError,
        models::{Age, Email, NewUser, PartialUserUpdate},
        services::CoreServices,
    };
    use uuid::Uuid;

    use crate::grpc::user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest, User as ProtoUser,
    };

    fn to_proto(user: hex_play_core::models::User) -> ProtoUser {
        ProtoUser {
            id: user.id,
            token: user.token.to_string(),
            name: user.name,
            email: user.email.to_string(),
            age: user.age.value() as i32,
        }
    }

    pub(crate) async fn create(core_services: &CoreServices, request: CreateUserRequest) -> Result<ProtoUser, Error> {
        let new_user = NewUser {
            name: request.name,
            email: Email::new(request.email)?,
            age: Age::new(request.age as i16)?,
        };
        let user = core_services.user_service.add_user(new_user).await?;
        Ok(to_proto(user))
    }

    pub(crate) async fn get(core_services: &CoreServices, request: GetUserRequest) -> Result<ProtoUser, Error> {
        let user = core_services
            .user_service
            .find_by_id(request.id)
            .await?
            .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;
        Ok(to_proto(user))
    }

    pub(crate) async fn get_by_token(core_services: &CoreServices, request: GetUserByTokenRequest) -> Result<ProtoUser, Error> {
        let token = request.token.parse::<Uuid>().map_err(|e| Error::InvalidUuid(e.to_string()))?;
        let user = core_services
            .user_service
            .find_by_token(token)
            .await?
            .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;
        Ok(to_proto(user))
    }

    pub(crate) async fn update(core_services: &CoreServices, request: UpdateUserRequest) -> Result<ProtoUser, Error> {
        let mut user = core_services
            .user_service
            .find_by_id(request.id)
            .await?
            .ok_or(Error::RepositoryError(RepositoryError::NotFound))?;

        let update = PartialUserUpdate {
            name: request.name,
            email: request.email.map(Email::new).transpose()?,
            age: request.age.map(|a| Age::new(a as i16)).transpose()?,
        };
        update.apply_to(&mut user);

        let user = core_services.user_service.update_user(user).await?;
        Ok(to_proto(user))
    }

    pub(crate) async fn delete(core_services: &CoreServices, request: DeleteUserRequest) -> Result<ProtoUser, Error> {
        let user = core_services.user_service.delete_user(request.id).await?;
        Ok(to_proto(user))
    }

    pub(crate) async fn list(core_services: &CoreServices, request: ListUsersRequest) -> Result<ListUsersResponse, Error> {
        let users = core_services
            .user_service
            .list_users(request.start_id, request.page_size)
            .await?
            .into_iter()
            .map(to_proto)
            .collect();
        Ok(ListUsersResponse { users })
    }
}

#[cfg(test)]
mod tests {
    use hex_play_core::{
        Error, RepositoryError,
        models::User,
        test_support::{MockUserService, create_arc_core_services_with_mock, create_core_services_with_mock},
    };
    use tonic::{Code, Request};
    use uuid::Uuid;

    use super::{GrpcUserService, handler};
    use crate::grpc::user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, UpdateUserRequest, user_service_server::UserService,
    };

    // ===================
    // Test Helpers
    // ===================
    fn create_test_service(mock: MockUserService) -> GrpcUserService {
        GrpcUserService::new(create_arc_core_services_with_mock(mock))
    }

    // ===================
    // Tests: handler::create
    // ===================
    #[tokio::test]
    async fn test_handler_create_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_add_user_result(Ok(user.clone()));
        let core_services = create_core_services_with_mock(mock);

        let request = CreateUserRequest {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        };

        let result = handler::create(&core_services, request).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "John Doe");
        assert_eq!(result.email, "john@example.com");
        assert_eq!(result.age, 30);
    }

    #[tokio::test]
    async fn test_handler_create_with_zero_age() {
        let user = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserService::default().with_add_user_result(Ok(user));
        let core_services = create_core_services_with_mock(mock);

        let request = CreateUserRequest {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 0,
        };

        let result = handler::create(&core_services, request).await.unwrap();

        assert_eq!(result.age, 0);
    }

    #[tokio::test]
    async fn test_handler_create_constraint_violation() {
        let mock = MockUserService::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate email".into()))));
        let core_services = create_core_services_with_mock(mock);

        let request = CreateUserRequest {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        };

        let result = handler::create(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: handler::get
    // ===================
    #[tokio::test]
    async fn test_handler_get_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_find_by_id_result(Ok(Some(user)));
        let core_services = create_core_services_with_mock(mock);

        let request = GetUserRequest { id: 1 };

        let result = handler::get(&core_services, request).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "John Doe");
        assert_eq!(result.email, "john@example.com");
        assert_eq!(result.age, 30);
    }

    #[tokio::test]
    async fn test_handler_get_not_found() {
        let mock = MockUserService::default().with_find_by_id_result(Ok(None));
        let core_services = create_core_services_with_mock(mock);

        let request = GetUserRequest { id: 999 };

        let result = handler::get(&core_services, request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::RepositoryError(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn test_handler_get_invalid_id() {
        let mock = MockUserService::default().with_find_by_id_result(Err(Error::InvalidId(-1)));
        let core_services = create_core_services_with_mock(mock);

        let request = GetUserRequest { id: -1 };

        let result = handler::get(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: handler::get_by_token
    // ===================
    #[tokio::test]
    async fn test_handler_get_by_token_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let token = user.token;
        let mock = MockUserService::default().with_find_by_token_result(Ok(Some(user)));
        let core_services = create_core_services_with_mock(mock);

        let request = GetUserByTokenRequest { token: token.to_string() };

        let result = handler::get_by_token(&core_services, request).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.token, token.to_string());
    }

    #[tokio::test]
    async fn test_handler_get_by_token_not_found() {
        let mock = MockUserService::default().with_find_by_token_result(Ok(None));
        let core_services = create_core_services_with_mock(mock);

        let token = Uuid::new_v4();
        let request = GetUserByTokenRequest { token: token.to_string() };

        let result = handler::get_by_token(&core_services, request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_get_by_token_invalid_uuid() {
        let mock = MockUserService::default();
        let core_services = create_core_services_with_mock(mock);

        let request = GetUserByTokenRequest { token: "not-a-uuid".into() };

        let result = handler::get_by_token(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: handler::update
    // ===================
    #[tokio::test]
    async fn test_handler_update_success() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let updated = User::test_with_age(1, "John Updated", "john@example.com", 30);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let core_services = create_core_services_with_mock(mock);

        let request = UpdateUserRequest {
            id: 1,
            name: Some("John Updated".into()),
            email: None,
            age: None,
        };

        let result = handler::update(&core_services, request).await.unwrap();

        assert_eq!(result.name, "John Updated");
        assert_eq!(result.age, 30);
    }

    #[tokio::test]
    async fn test_handler_update_email_only() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 25);
        let updated = User::test_with_age(1, "John Doe", "john.new@example.com", 25);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let core_services = create_core_services_with_mock(mock);

        let request = UpdateUserRequest {
            id: 1,
            name: None,
            email: Some("john.new@example.com".into()),
            age: None,
        };

        let result = handler::update(&core_services, request).await.unwrap();

        assert_eq!(result.email, "john.new@example.com");
    }

    #[tokio::test]
    async fn test_handler_update_age_only() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let updated = User::test_with_age(1, "John Doe", "john@example.com", 31);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let core_services = create_core_services_with_mock(mock);

        let request = UpdateUserRequest {
            id: 1,
            name: None,
            email: None,
            age: Some(31),
        };

        let result = handler::update(&core_services, request).await.unwrap();

        assert_eq!(result.age, 31);
    }

    #[tokio::test]
    async fn test_handler_update_not_found() {
        let mock = MockUserService::default().with_find_by_id_result(Ok(None));
        let core_services = create_core_services_with_mock(mock);

        let request = UpdateUserRequest {
            id: 999,
            name: Some("Updated".into()),
            email: None,
            age: None,
        };

        let result = handler::update(&core_services, request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_update_conflict() {
        let existing = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Err(Error::RepositoryError(RepositoryError::Conflict)));
        let core_services = create_core_services_with_mock(mock);

        let request = UpdateUserRequest {
            id: 1,
            name: Some("Updated".into()),
            email: None,
            age: None,
        };

        let result = handler::update(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: handler::delete
    // ===================
    #[tokio::test]
    async fn test_handler_delete_success() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_delete_user_result(Ok(user));
        let core_services = create_core_services_with_mock(mock);

        let request = DeleteUserRequest { id: 1 };

        let result = handler::delete(&core_services, request).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "John Doe");
        assert_eq!(result.age, 30);
    }

    #[tokio::test]
    async fn test_handler_delete_not_found() {
        let mock = MockUserService::default().with_delete_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let core_services = create_core_services_with_mock(mock);

        let request = DeleteUserRequest { id: 999 };

        let result = handler::delete(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: handler::list
    // ===================
    #[tokio::test]
    async fn test_handler_list_success() {
        let users = vec![
            User::test_with_age(1, "John Doe", "john@example.com", 30),
            User::test_with_age(2, "Jane Doe", "jane@example.com", 25),
        ];
        let mock = MockUserService::default().with_list_users_result(Ok(users));
        let core_services = create_core_services_with_mock(mock);

        let request = ListUsersRequest {
            start_id: None,
            page_size: None,
        };

        let result = handler::list(&core_services, request).await.unwrap();

        assert_eq!(result.users.len(), 2);
        assert_eq!(result.users[0].name, "John Doe");
        assert_eq!(result.users[1].name, "Jane Doe");
    }

    #[tokio::test]
    async fn test_handler_list_empty() {
        let mock = MockUserService::default().with_list_users_result(Ok(vec![]));
        let core_services = create_core_services_with_mock(mock);

        let request = ListUsersRequest {
            start_id: None,
            page_size: None,
        };

        let result = handler::list(&core_services, request).await.unwrap();

        assert!(result.users.is_empty());
    }

    #[tokio::test]
    async fn test_handler_list_with_pagination() {
        let users = vec![User::test(5, "User Five", "five@example.com")];
        let mock = MockUserService::default().with_list_users_result(Ok(users));
        let core_services = create_core_services_with_mock(mock);

        let request = ListUsersRequest {
            start_id: Some(5),
            page_size: Some(10),
        };

        let result = handler::list(&core_services, request).await.unwrap();

        assert_eq!(result.users.len(), 1);
    }

    #[tokio::test]
    async fn test_handler_list_invalid_start_id() {
        let mock = MockUserService::default().with_list_users_result(Err(Error::InvalidId(-1)));
        let core_services = create_core_services_with_mock(mock);

        let request = ListUsersRequest {
            start_id: Some(-1),
            page_size: None,
        };

        let result = handler::list(&core_services, request).await;

        assert!(result.is_err());
    }

    // ===================
    // Tests: GrpcUserService trait implementation
    // ===================
    #[tokio::test]
    async fn test_grpc_service_create() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_add_user_result(Ok(user));
        let service = create_test_service(mock);

        let request = Request::new(CreateUserRequest {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        });

        let response = service.create(request).await.unwrap();
        let proto_user = response.into_inner();

        assert_eq!(proto_user.id, 1);
        assert_eq!(proto_user.name, "John Doe");
    }

    #[tokio::test]
    async fn test_grpc_service_create_error_maps_to_status() {
        let mock = MockUserService::default().with_add_user_result(Err(Error::RepositoryError(RepositoryError::Constraint("duplicate".into()))));
        let service = create_test_service(mock);

        let request = Request::new(CreateUserRequest {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            age: 30,
        });

        let result = service.create(request).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_grpc_service_get() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_find_by_id_result(Ok(Some(user)));
        let service = create_test_service(mock);

        let request = Request::new(GetUserRequest { id: 1 });

        let response = service.get(request).await.unwrap();
        let proto_user = response.into_inner();

        assert_eq!(proto_user.id, 1);
    }

    #[tokio::test]
    async fn test_grpc_service_get_not_found_maps_to_status() {
        let mock = MockUserService::default().with_find_by_id_result(Ok(None));
        let service = create_test_service(mock);

        let request = Request::new(GetUserRequest { id: 999 });

        let result = service.get(request).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn test_grpc_service_get_by_token() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let token = user.token;
        let mock = MockUserService::default().with_find_by_token_result(Ok(Some(user)));
        let service = create_test_service(mock);

        let request = Request::new(GetUserByTokenRequest { token: token.to_string() });

        let response = service.get_by_token(request).await.unwrap();
        let proto_user = response.into_inner();

        assert_eq!(proto_user.token, token.to_string());
    }

    #[tokio::test]
    async fn test_grpc_service_update() {
        let existing = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let updated = User::test_with_age(1, "John Updated", "john@example.com", 30);
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Ok(updated));
        let service = create_test_service(mock);

        let request = Request::new(UpdateUserRequest {
            id: 1,
            name: Some("John Updated".into()),
            email: None,
            age: None,
        });

        let response = service.update(request).await.unwrap();
        let proto_user = response.into_inner();

        assert_eq!(proto_user.name, "John Updated");
    }

    #[tokio::test]
    async fn test_grpc_service_update_conflict_maps_to_status() {
        let existing = User::test(1, "John Doe", "john@example.com");
        let mock = MockUserService::default()
            .with_find_by_id_result(Ok(Some(existing)))
            .with_update_user_result(Err(Error::RepositoryError(RepositoryError::Conflict)));
        let service = create_test_service(mock);

        let request = Request::new(UpdateUserRequest {
            id: 1,
            name: Some("Updated".into()),
            email: None,
            age: None,
        });

        let result = service.update(request).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::AlreadyExists);
    }

    #[tokio::test]
    async fn test_grpc_service_delete() {
        let user = User::test_with_age(1, "John Doe", "john@example.com", 30);
        let mock = MockUserService::default().with_delete_user_result(Ok(user));
        let service = create_test_service(mock);

        let request = Request::new(DeleteUserRequest { id: 1 });

        let response = service.delete(request).await.unwrap();
        let proto_user = response.into_inner();

        assert_eq!(proto_user.id, 1);
    }

    #[tokio::test]
    async fn test_grpc_service_delete_not_found_maps_to_status() {
        let mock = MockUserService::default().with_delete_user_result(Err(Error::RepositoryError(RepositoryError::NotFound)));
        let service = create_test_service(mock);

        let request = Request::new(DeleteUserRequest { id: 999 });

        let result = service.delete(request).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn test_grpc_service_list() {
        let users = vec![
            User::test_with_age(1, "John Doe", "john@example.com", 30),
            User::test_with_age(2, "Jane Doe", "jane@example.com", 25),
        ];
        let mock = MockUserService::default().with_list_users_result(Ok(users));
        let service = create_test_service(mock);

        let request = Request::new(ListUsersRequest {
            start_id: None,
            page_size: None,
        });

        let response = service.list(request).await.unwrap();
        let list_response = response.into_inner();

        assert_eq!(list_response.users.len(), 2);
    }
}

/// Client-side API (returns core domain types)
pub mod api {
    use hex_play_core::{
        Error,
        models::{Age, Email, User},
    };
    use uuid::Uuid;

    use crate::grpc::user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, UpdateUserRequest, User as ProtoUser,
        user_service_client::UserServiceClient,
    };

    fn from_proto(proto: ProtoUser) -> Result<User, Error> {
        Ok(User {
            id: proto.id,
            token: proto.token.parse::<Uuid>().map_err(|e| Error::InvalidUuid(e.to_string()))?,
            name: proto.name,
            email: Email::new(proto.email)?,
            age: Age::new(proto.age as i16)?,
            ..Default::default()
        })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn create(name: String, email: String, age: i16) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(CreateUserRequest { name, email, age: age as i32 });
        let response = client.create(request).await.map_err(|e| Error::GrpcClientError(e.to_string()))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get(id: i64) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(GetUserRequest { id });
        let response = client.get(request).await.map_err(|e| Error::GrpcClientError(e.to_string()))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get_by_token(token: Uuid) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(GetUserByTokenRequest { token: token.to_string() });
        let response = client
            .get_by_token(request)
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?
            .into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn update(id: i64, name: Option<String>, email: Option<String>, age: Option<i16>) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(UpdateUserRequest {
            id,
            name,
            email,
            age: age.map(|a| a as i32),
        });
        let response = client.update(request).await.map_err(|e| Error::GrpcClientError(e.to_string()))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn delete(id: i64) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(DeleteUserRequest { id });
        let response = client.delete(request).await.map_err(|e| Error::GrpcClientError(e.to_string()))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn list(start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001")
            .await
            .map_err(|e| Error::GrpcClientError(e.to_string()))?;
        let request = tonic::Request::new(ListUsersRequest { start_id, page_size });
        let response = client.list(request).await.map_err(|e| Error::GrpcClientError(e.to_string()))?.into_inner();
        response.users.into_iter().map(from_proto).collect()
    }
}
