use super::{default_page, default_page_size, ApiTags, AppState};
use crate::oai_schema;
use crate::{auth::JWTAuthorization, repository::Repository};
use poem::web::Data;
use poem::Result;
use poem_openapi::param::{Path, Query};
use poem_openapi::{payload::Json, OpenApi};

pub struct SchemaService;

#[OpenApi(prefix_path = "/schema", tag = "ApiTags::Schema")]
impl SchemaService {
    /// 创建数据模型
    #[oai(path = "/", method = "post")]
    async fn create_schema(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        body: Json<oai_schema::CreateSchema>,
    ) -> Result<Json<oai_schema::Schema>> {
        let schema = state.repo.create_schema(&account.0, &body).await?;
        Ok(Json(schema.into()))
    }

    /// 查询数据模型列表
    #[oai(path = "/", method = "get")]
    async fn list_schema(
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
        /// 模糊查询数据模型名称
        q: Query<Option<String>>,
    ) -> Result<Json<oai_schema::Schemas>> {
        let (schemas, total) = state
            .repo
            .list_schema(&account.0, page.0, page_size.0, id_in.clone(), q.clone())
            .await?;
        Ok(Json(oai_schema::Schemas {
            results: schemas.into_iter().map(|schema| schema.into()).collect(),
            total,
        }))
    }

    /// 查询数据模型详情
    #[oai(path = "/:schema_id", method = "get")]
    async fn get_schema(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
    ) -> Result<Json<oai_schema::SchemaWithFields>> {
        let schema = state
            .repo
            .get_schema_with_related(&account.0, &schema_id)
            .await?;
        Ok(Json(schema.into()))
    }

    /// 更新数据模型
    #[oai(path = "/:schema_id", method = "patch")]
    async fn update_schema(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
        body: Json<oai_schema::UpdateSchema>,
    ) -> Result<Json<oai_schema::Schema>> {
        let schema = state
            .repo
            .update_schema(&account.0, &schema_id, &body)
            .await?;
        Ok(Json(schema.into()))
    }

    /// 删除数据模型
    #[oai(path = "/:schema_id", method = "delete")]
    async fn delete_schema(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
    ) -> Result<()> {
        state.repo.delete_schema(&account.0, &schema_id).await?;
        Ok(())
    }

    /// 数据模型添加字段
    #[oai(path = "/:schema_id/field", method = "post")]
    async fn create_field(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
        body: Json<oai_schema::CreateField>,
    ) -> Result<Json<oai_schema::Field>> {
        let field = state
            .repo
            .create_field(&account.0, &schema_id, &body)
            .await?;
        Ok(Json(field.into()))
    }

    /// 数据模型更新字段
    #[oai(path = "/:schema_id/field/:identifier", method = "patch")]
    async fn update_field(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
        identifier: Path<String>,
        body: Json<oai_schema::UpdateField>,
    ) -> Result<Json<oai_schema::Field>> {
        let field = state
            .repo
            .update_field(&account.0, &schema_id, &identifier, &body)
            .await?;
        Ok(Json(field.into()))
    }
    /// 数据模型删除字段
    #[oai(path = "/:schema_id/field/:identifier", method = "delete")]
    async fn delete_field(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        schema_id: Path<String>,
        identifier: Path<String>,
    ) -> Result<()> {
        state
            .repo
            .delete_field(&account.0, &schema_id, &identifier)
            .await?;
        Ok(())
    }
}
