use super::{default_page, default_page_size, ApiTags, AppState};
use crate::oai_schema;
use crate::{auth::JWTAuthorization, errors::NeoiotError, repository::Repository};
use poem::web::Data;
use poem::Result;
use poem_openapi::param::{Path, Query};
use poem_openapi::{payload::Json, OpenApi};

pub struct AccountService;

#[OpenApi(prefix_path = "/admin/account", tag = "ApiTags::Account")]
impl AccountService {
    /// 创建账号
    #[oai(path = "/", method = "post")]
    async fn create_account(
        &self,
        account: JWTAuthorization,
        state: Data<&AppState>,
        body: Json<oai_schema::CreateAccount>,
    ) -> Result<Json<oai_schema::Account>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(NeoiotError::PermissionDenied.into());
        }
        let new_account = state.repo.create_account(&body).await?;
        Ok(Json(new_account.into()))
    }

    /// 查询账号列表
    #[oai(path = "/", method = "get")]
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
    ) -> Result<Json<oai_schema::Accounts>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(NeoiotError::PermissionDenied.into());
        }
        let (schema, total) = state
            .repo
            .list_account(page.0, page_size.0, id_in.clone(), q.clone())
            .await?;
        Ok(Json(oai_schema::Accounts {
            results: schema.into_iter().map(|account| account.into()).collect(),
            total,
        }))
    }

    /// 查询账号详情
    #[oai(path = "/:account_id", method = "get")]
    async fn get_account(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 要获取的账户ID
        account_id: Path<String>,
    ) -> Result<Json<oai_schema::Account>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(NeoiotError::PermissionDenied.into());
        }
        let account = state.repo.get_account(&account_id).await?;
        Ok(Json(account.into()))
    }

    /// 更新账号信息
    #[oai(path = "/:account_id", method = "patch")]
    async fn update_account(
        &self,
        account: JWTAuthorization,
        state: Data<&AppState>,
        /// 要更新的账户ID
        account_id: Path<String>,
        body: Json<oai_schema::UpdateAccount>,
    ) -> Result<Json<oai_schema::Account>> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(NeoiotError::PermissionDenied.into());
        }
        let account = state.repo.update_account(&account_id, &body).await?;
        Ok(Json(account.into()))
    }

    /// 删除账号
    #[oai(path = "/:account_id", method = "delete")]
    async fn delete_account(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 要删除的账户ID
        account_id: Path<String>,
    ) -> Result<()> {
        let account = state.repo.get_account(&account.0).await?;
        if !account.is_superuser {
            return Err(NeoiotError::PermissionDenied.into());
        }
        state.repo.delete_account(&account_id).await?;
        Ok(())
    }
}
