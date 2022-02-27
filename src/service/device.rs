use super::{default_page, default_page_size, ApiTags, AppState};
use crate::oai_schema;
use crate::{auth::JWTAuthorization, repository::Repository};
use poem::web::Data;
use poem::Result;
use poem_openapi::param::{Path, Query};
use poem_openapi::{payload::Json, OpenApi};

pub struct DeviceService;

#[OpenApi(prefix_path = "/device", tag = "ApiTags::Device")]
impl DeviceService {
    /// 创建设备
    #[oai(path = "/", method = "post")]
    async fn create_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        body: Json<oai_schema::CreateDevice>,
    ) -> Result<Json<oai_schema::DeviceWithLables>> {
        let device = state.repo.create_device(&account.0, &body).await?;
        Ok(Json(device.into()))
    }

    /// 查询设备列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn list_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 根据id筛选
        id_in: Query<Option<Vec<String>>>,
        /// 模糊查询设备名称
        q: Query<Option<String>>,
        /// 筛选包含标签的设备
        labels_in: Query<Option<Vec<String>>>,
        /// 第几页
        #[oai(default = "default_page")]
        page: Query<usize>,
        /// 每页条目数
        #[oai(default = "default_page_size")]
        page_size: Query<usize>,
    ) -> Result<Json<oai_schema::Devices>> {
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
        Ok(Json(oai_schema::Devices {
            results: devices.into_iter().map(|device| device.into()).collect(),
            total,
        }))
    }

    /// 获取设备详情
    #[oai(path = "/:device_id", method = "get")]
    async fn get_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
    ) -> Result<Json<oai_schema::DeviceWithLables>> {
        let device = state
            .repo
            .get_device_with_labels(&account.0, &device_id)
            .await?;
        Ok(Json(device.into()))
    }
    /// 更新设备信息
    #[oai(path = "/:device_id", method = "patch")]
    async fn update_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
        body: Json<oai_schema::UpdateDevice>,
    ) -> Result<Json<oai_schema::DeviceWithLables>> {
        let device = state
            .repo
            .update_device(&account.0, &device_id, &body)
            .await?;
        Ok(Json(device.into()))
    }

    /// 删除设备
    #[oai(path = "/:device_id", method = "delete")]
    async fn delete_device(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
    ) -> Result<()> {
        state.repo.delete_device(&account.0, &device_id).await?;
        Ok(())
    }
    /// 向设备发送指令
    #[oai(path = "/:device_id/command", method = "post")]
    async fn send_command_to_deivce(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        device_id: Path<String>,
        req: Json<oai_schema::SendCommandToDevice>,
    ) -> Result<oai_schema::CommandResponse> {
        let response = state
            .repo
            .send_command_to_device(&account.0, &device_id, &req)
            .await?;
        Ok(response)
    }

    /// 查询设备连接信息列表
    #[oai(path = "/:device_id/connections", method = "get")]
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
    ) -> Result<Json<oai_schema::DeviceConnections>> {
        let result = state
            .repo
            .list_device_connections(&account.0, &device_id, page.0, page_size.0)
            .await?;
        Ok(Json(result.into()))
    }
}
