use super::{auth::JWTAuthorization, AppState};
use crate::config::SETTINGS;
use crate::entity::accounts::AccountListResp;
use crate::entity::{accounts, device_connections, devices, mappings, properties};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jwt_simple::prelude::*;
use poem::http::StatusCode;
use poem::web::Data;
use poem::{Error, Result};
use poem_openapi::param::{Path, Query};
use poem_openapi::{payload::Json, OpenApi, Tags};

#[derive(Default)]
pub(crate) struct APIv1;

#[derive(Tags)]
enum ApiTags {
    /// Auth相关API
    Auth,
    /// 账号相关API
    Account,
    /// 设备相关API
    Device,
    /// 映射集相关API
    Mapping,
}

const fn default_page() -> usize {
    1
}
const fn default_page_size() -> usize {
    10
}

#[OpenApi]
impl APIv1 {
    /// 获取Token
    #[oai(path = "/auth/token/obtain", method = "post", tag = "ApiTags::Auth")]
    async fn obtain_token(
        &self,
        state: Data<&AppState>,
        data: Json<accounts::Login>,
    ) -> Result<Json<accounts::LoginResp>> {
        let account = state.repo.get_account_by_email(&data.email).await?;
        if !verify_password(&data.password, &account.password) {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        let claims =
            Claims::create(jwt_simple::prelude::Duration::from_days(1)).with_subject(account.email);
        let key = HS256Key::from_bytes(SETTINGS.read().unwrap().secret.as_bytes());
        let token = key.authenticate(claims)?;
        Ok(Json(accounts::LoginResp { token }))
    }

    /// 创建账号(需要管理员权限)
    #[oai(path = "/admin/account", method = "post", tag = "ApiTags::Account")]
    async fn create_account(
        &self,
        account: JWTAuthorization,
        state: Data<&AppState>,
        body: Json<accounts::AccountCreateReq>,
    ) -> Result<Json<accounts::AccountResp>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        let new_account = state.repo.create_account(&body).await?;
        Ok(Json(new_account.into()))
    }

    /// 查询账号列表(需要管理员权限)
    #[oai(path = "/admin/account", method = "get", tag = "ApiTags::Account")]
    async fn list_account(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 第几页
        #[oai(default = "default_page")]
        page: Query<usize>,
        /// 根据id批量查询
        id_in: Query<Option<Vec<String>>>,
        /// 每页条目数
        #[oai(default = "default_page_size")]
        page_size: Query<usize>,
        /// 模糊查询账号名称
        q: Query<Option<String>>,
    ) -> Result<Json<accounts::AccountListResp>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        let (accounts, total) = state
            .repo
            .list_account(page.0, page_size.0, id_in.clone(), q.clone())
            .await?;
        Ok(Json(AccountListResp {
            results: accounts.into_iter().map(|account| account.into()).collect(),
            total,
        }))
    }

    /// 查询账号详情(需要管理员权限)
    #[oai(
        path = "/admin/account/:account_id",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn get_account(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 要获取的账户ID
        account_id: Path<String>,
    ) -> Result<Json<accounts::AccountResp>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        let account = state.repo.get_account(&account_id).await?;
        Ok(Json(account.into()))
    }

    /// 更新账号信息(需要管理员权限)
    #[oai(
        path = "/admin/account/:account_id",
        method = "patch",
        tag = "ApiTags::Account"
    )]
    async fn update_account(
        &self,
        account: JWTAuthorization,
        state: Data<&AppState>,
        /// 要更新的账户ID
        account_id: Path<String>,
        body: Json<accounts::AccountUpdateReq>,
    ) -> Result<Json<accounts::AccountResp>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        let account = state.repo.update_account(&account_id, &body).await?;
        Ok(Json(account.into()))
    }

    /// 删除账号(需要管理员权限)
    #[oai(
        path = "/admin/account/:account_id",
        method = "delete",
        tag = "ApiTags::Account"
    )]
    async fn delete_account(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 要删除的账户ID
        account_id: Path<String>,
    ) -> Result<()> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }
        state.repo.delete_account(&account_id).await?;
        Ok(())
    }

    /// 创建设备
    #[oai(path = "/device", method = "post", tag = "ApiTags::Device")]
    async fn create_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        body: Json<devices::DeviceCreateReq>,
    ) -> Result<Json<devices::DeviceResp>> {
        let device = state.repo.create_device(&account.0, &body).await?;
        Ok(Json(device.into()))
    }

    /// 查询设备列表
    #[oai(path = "/device", method = "get", tag = "ApiTags::Device")]
    #[allow(clippy::too_many_arguments)]
    async fn list_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 第几页
        #[oai(default = "default_page")]
        page: Query<usize>,
        /// 根据id筛选
        id_in: Query<Option<Vec<String>>>,
        /// 筛选包含标签的设备
        labels_in: Query<Option<Vec<String>>>,
        /// 每页条目数
        #[oai(default = "default_page_size")]
        page_size: Query<usize>,
        /// 模糊查询设备名称
        q: Query<Option<String>>,
    ) -> Result<Json<devices::DeviceListResp>> {
        let (devices, total) = state
            .repo
            .list_device(
                Some(&account.0),
                page.0,
                page_size.0,
                id_in.clone(),
                labels_in.clone(),
                q.clone(),
            )
            .await?;
        Ok(Json(devices::DeviceListResp {
            results: devices.into_iter().map(|device| device.into()).collect(),
            total,
        }))
    }

    /// 获取设备详情
    #[oai(path = "/device/:device_id", method = "get", tag = "ApiTags::Device")]
    async fn get_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
    ) -> Result<Json<devices::DeviceResp>> {
        let device = state
            .repo
            .get_device_with_labels(&account.0, &device_id)
            .await?;
        Ok(Json(device.into()))
    }
    /// 更新设备信息
    #[oai(path = "/device/:device_id", method = "patch", tag = "ApiTags::Device")]
    async fn update_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
        body: Json<devices::DeviceUpdateReq>,
    ) -> Result<Json<devices::DeviceResp>> {
        let device = state
            .repo
            .update_device(&account.0, &device_id, &body)
            .await?;
        Ok(Json(device.into()))
    }

    /// 删除设备
    #[oai(
        path = "/device/:device_id",
        method = "delete",
        tag = "ApiTags::Device"
    )]
    async fn delete_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
    ) -> Result<()> {
        state.repo.delete_device(&account.0, &device_id).await?;
        Ok(())
    }

    /// 查询设备连接信息列表
    #[oai(
        path = "/device/:device_id/connections",
        method = "get",
        tag = "ApiTags::Device"
    )]
    async fn list_device_connections(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
        /// 第几页
        #[oai(default = "default_page")]
        page: Query<usize>,
        /// 每页条目数
        #[oai(default = "default_page_size")]
        page_size: Query<usize>,
    ) -> Result<Json<device_connections::DeviceConnectionsListResp>> {
        let result = state
            .repo
            .list_device_connections(&account.0, &device_id, page.0, page_size.0)
            .await?;
        Ok(Json(result.into()))
    }

    /// 创建映射集
    #[oai(path = "/mapping", method = "post", tag = "ApiTags::Mapping")]
    async fn create_mapping(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        body: Json<mappings::MappingCreateReq>,
    ) -> Result<Json<mappings::MappingResp>> {
        let mapping = state.repo.create_mapping(&account.0, &body).await?;
        Ok(Json(mapping.into()))
    }

    /// 查询映射集列表
    #[oai(path = "/mapping", method = "get", tag = "ApiTags::Mapping")]
    async fn list_mapping(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 第几页
        #[oai(default = "default_page")]
        page: Query<usize>,
        /// 根据id批量查询
        id_in: Query<Option<Vec<String>>>,
        /// 每页条目数
        #[oai(default = "default_page_size")]
        page_size: Query<usize>,
        /// 模糊查询映射集名称
        q: Query<Option<String>>,
    ) -> Result<Json<mappings::MappingListResp>> {
        let (mappings, total) = state
            .repo
            .list_mapping(&account.0, page.0, page_size.0, id_in.clone(), q.clone())
            .await?;
        Ok(Json(mappings::MappingListResp {
            results: mappings.into_iter().map(|mapping| mapping.into()).collect(),
            total,
        }))
    }

    /// 查询映射集详情
    #[oai(
        path = "/mapping/:mapping_id",
        method = "get",
        tag = "ApiTags::Mapping"
    )]
    async fn get_mapping(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        mapping_id: Path<String>,
    ) -> Result<Json<mappings::MappingResp>> {
        let mapping = state.repo.get_mapping(&account.0, &mapping_id).await?;
        Ok(Json(mapping.into()))
    }

    /// 更新映射集
    #[oai(
        path = "/mapping/:mapping_id",
        method = "patch",
        tag = "ApiTags::Mapping"
    )]
    async fn update_mapping(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        mapping_id: Path<String>,
        body: Json<mappings::MappingUpdateReq>,
    ) -> Result<Json<mappings::MappingResp>> {
        let mapping = state
            .repo
            .update_mapping(&account.0, &mapping_id, &body)
            .await?;
        Ok(Json(mapping.into()))
    }

    /// 删除映射集
    #[oai(
        path = "/mapping/:mapping_id",
        method = "delete",
        tag = "ApiTags::Mapping"
    )]
    async fn delete_mapping(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        mapping_id: Path<String>,
    ) -> Result<()> {
        state.repo.delete_mapping(&account.0, &mapping_id).await?;
        Ok(())
    }

    /// 映射集添加属性
    #[oai(
        path = "/mapping/:mapping_id/property",
        method = "post",
        tag = "ApiTags::Mapping"
    )]
    async fn create_property(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        mapping_id: Path<String>,
        body: Json<properties::PropertyCreateReq>,
    ) -> Result<Json<properties::PropertyResp>> {
        let property = state
            .repo
            .create_property(&account.0, &mapping_id, &body)
            .await?;
        Ok(Json(property.into()))
    }

    /// 映射集更新属性
    #[oai(
        path = "/mapping/:mapping_id/property/:identifier",
        method = "patch",
        tag = "ApiTags::Mapping"
    )]
    async fn update_property(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        mapping_id: Path<String>,
        identifier: Path<String>,
        body: Json<properties::PropertyUpdateReq>,
    ) -> Result<Json<properties::PropertyResp>> {
        let property = state
            .repo
            .update_property(&account.0, &mapping_id, &identifier, &body)
            .await?;
        Ok(Json(property.into()))
    }

    // async fn send_command(
    //     &self,
    //     request: Request<SendCommandRequest>,
    // ) -> Result<Response<SendCommandResponse>, Status> {
    //     todo!()
    // }
    // async fn send_command_by_tag(
    //     &self,
    //     request: Request<SendCommandRequestByTag>,
    // ) -> Result<Response<SendCommandResponse>, Status> {
    //     todo!()
    // }
    // async fn query_latest(
    //     &self,
    //     request: Request<QueryLatestRequest>,
    // ) -> Result<Response<::prost_types::Struct>, Status> {
    //     todo!()
    // }
    // async fn query_batch(
    //     &self,
    //     request: Request<QueryBatchRequest>,
    // ) -> Result<Response<QueryBatchResponse>, Status> {
    //     todo!()
    // }
}

fn verify_password(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let hashed = PasswordHash::new(hash).unwrap();
    argon2.verify_password(password.as_bytes(), &hashed).is_ok()
}
