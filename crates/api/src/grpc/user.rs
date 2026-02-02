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
    use hex_play_core::{Error, RepositoryError, models::NewUser, services::CoreServices};
    use uuid::Uuid;

    use crate::grpc::user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest, User as ProtoUser,
    };

    fn to_proto(user: hex_play_core::models::User) -> ProtoUser {
        ProtoUser {
            id: user.id,
            token: user.token.to_string(),
            name: user.name,
            email: user.email,
            age: user.age as i32,
        }
    }

    pub(crate) async fn create(core_services: &CoreServices, request: CreateUserRequest) -> Result<ProtoUser, Error> {
        let new_user = NewUser {
            name: request.name,
            email: request.email,
            age: request.age as i16,
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
        let token = request.token.parse::<Uuid>().map_err(|e| Error::Message(format!("Invalid UUID: {}", e)))?;
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

        if let Some(name) = request.name {
            user.name = name;
        }
        if let Some(email) = request.email {
            user.email = email;
        }
        if let Some(age) = request.age {
            user.age = age as i16;
        }

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

/// Client-side API (returns core domain types)
pub mod api {
    use hex_play_core::{Error, models::User};
    use uuid::Uuid;

    use crate::grpc::user_proto::{
        CreateUserRequest, DeleteUserRequest, GetUserByTokenRequest, GetUserRequest, ListUsersRequest, UpdateUserRequest, User as ProtoUser,
        user_service_client::UserServiceClient,
    };

    fn from_proto(proto: ProtoUser) -> Result<User, Error> {
        Ok(User {
            id: proto.id,
            token: proto.token.parse::<Uuid>().map_err(|e| Error::Message(format!("Invalid UUID: {}", e)))?,
            name: proto.name,
            email: proto.email,
            age: proto.age as i16,
            ..Default::default()
        })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn create(name: String, email: String, age: i16) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(CreateUserRequest { name, email, age: age as i32 });
        let response = client.create(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get(id: i64) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(GetUserRequest { id });
        let response = client.get(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get_by_token(token: Uuid) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(GetUserByTokenRequest { token: token.to_string() });
        let response = client.get_by_token(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn update(id: i64, name: Option<String>, email: Option<String>, age: Option<i16>) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(UpdateUserRequest {
            id,
            name,
            email,
            age: age.map(|a| a as i32),
        });
        let response = client.update(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn delete(id: i64) -> Result<User, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(DeleteUserRequest { id });
        let response = client.delete(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        from_proto(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn list(start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        let mut client = UserServiceClient::connect("http://localhost:3001").await.map_err(|e| Error::Any(Box::new(e)))?;
        let request = tonic::Request::new(ListUsersRequest { start_id, page_size });
        let response = client.list(request).await.map_err(|e| Error::Any(Box::new(e)))?.into_inner();
        response.users.into_iter().map(from_proto).collect()
    }
}
