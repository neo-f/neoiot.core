use super::{ApiTags, AppState};
use crate::oai_schema;
use crate::{auth::JWTAuthorization, repository::Repository};
use poem::web::Data;
use poem::Result;
use poem_openapi::param::{Path, Query};
use poem_openapi::{payload::Json, OpenApi};

pub struct LabelService;

#[OpenApi(prefix_path = "/label", tag = "ApiTags::Label")]
impl LabelService {
    /// 查询标签列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn list_label(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        /// 模糊查询标签名称
        q: Query<Option<String>>,
    ) -> Result<Json<oai_schema::Labels>> {
        let labels = state.repo.list_labels(&account.0, q.0).await?;
        Ok(Json(oai_schema::Labels {
            results: labels.into_iter().map(|label| label.into()).collect(),
        }))
    }

    /// 更新标签信息
    #[oai(path = "/:label_id", method = "patch")]
    async fn update_label(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        label_id: Path<String>,
        body: Json<oai_schema::UpdateLabel>,
    ) -> Result<Json<oai_schema::Label>> {
        let label = state
            .repo
            .update_label(&account.0, &label_id, &body)
            .await?;
        Ok(Json(label.into()))
    }
    /// 删除标签
    #[oai(path = "/:label_id", method = "delete")]
    async fn delete_label(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        label_id: Path<String>,
    ) -> Result<()> {
        state.repo.delete_label(&account.0, &label_id).await?;
        Ok(())
    }
    /// 向包含标签的设备批量发送指令
    #[oai(path = "/:label_id/command", method = "post")]
    async fn send_command_to_deivce(
        &self,
        state: Data<&AppState>,
        account: JWTAuthorization,
        label_id: Path<String>,
        req: Json<oai_schema::SendCommandToDeviceBatch>,
    ) -> Result<oai_schema::CommandResponse> {
        let message_id = state
            .repo
            .send_command_to_label(&account.0, &label_id, &req)
            .await?;
        Ok(oai_schema::CommandResponse::new_async(message_id))
    }
}
