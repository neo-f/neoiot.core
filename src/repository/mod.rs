use crate::errors::Result;
use entity::prelude::*;
use poem::async_trait;

mod postgres;
pub use postgres::PostgresRepository;

use crate::io_schema::{self, SchemaModelWithRelated};

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    ////////////////////////////// 账号相关//////////////////////////////////////////////////////////
    /// 创建账号
    async fn create_account(&self, req: &io_schema::CreateAccount) -> Result<AccountModel>;
    /// 获取一个账号信息
    async fn get_account(&self, account_id: &str) -> Result<AccountModel>;
    /// 获取一个账号信息(通过邮箱)
    async fn get_account_by_email(&self, email: &str) -> Result<AccountModel>;
    /// 更新账号登录时间
    async fn after_account_logined(&self, email: &str) -> Result<()>;
    /// 获取账号列表
    async fn list_account(
        &self,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<AccountModel>, usize)>;
    /// 更新账号信息
    async fn update_account(
        &self,
        account_id: &str,
        req: &io_schema::UpdateAccount,
    ) -> Result<AccountModel>;
    /// 删除账号
    async fn delete_account(&self, account_id: &str) -> Result<()>;

    ////////////////////////////// 设备相关//////////////////////////////////////////////////////////
    /// 获取一条设备信息
    async fn get_device(&self, account_id: &str, device_id: &str) -> Result<DeviceModel>;
    /// 获取一条设备和Label信息
    async fn get_device_with_labels(
        &self,
        account_id: &str,
        device_id: &str,
    ) -> Result<io_schema::DeviceModelWithRelated>;
    /// 获取设备列表
    async fn list_device(
        &self,
        account_id: Option<&str>,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        labels_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<DeviceModel>, usize)>;
    /// 更新设备信息
    async fn update_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &io_schema::UpdateDevice,
    ) -> Result<io_schema::DeviceModelWithRelated>;
    /// 删除设备
    async fn delete_device(&self, account_id: &str, device_id: &str) -> Result<()>;
    /// 创建设备
    async fn create_device(
        &self,
        account_id: &str,
        req: &io_schema::CreateDevice,
    ) -> Result<io_schema::DeviceModelWithRelated>;
    /// 获取设备的连接信息
    async fn list_device_connections(
        &self,
        account_id: &str,
        device_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<DeviceConnectionModel>, usize)>;
    async fn send_command_to_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &io_schema::SendCommandToDevice,
    ) -> Result<io_schema::CommandResponse>;

    ////////////////////////////// 数据模型相关//////////////////////////////////////////////////////////
    /// 创建一个数据模型
    async fn create_schema(
        &self,
        account_id: &str,
        schema: &io_schema::CreateSchema,
    ) -> Result<SchemaModel>;
    /// 获取一个数据模型
    async fn get_schema_with_related(
        &self,
        account_id: &str,
        schema_id: &str,
    ) -> Result<SchemaModelWithRelated>;
    async fn get_schema(&self, account_id: &str, schema_id: &str) -> Result<SchemaModel>;
    /// 获取数据模型列表
    async fn list_schema(
        &self,
        account_id: &str,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<SchemaModel>, usize)>;
    /// 更新数据模型
    async fn update_schema(
        &self,
        account_id: &str,
        schema_id: &str,
        req: &io_schema::UpdateSchema,
    ) -> Result<SchemaModel>;
    /// 删除数据模型
    async fn delete_schema(&self, account_id: &str, schema_id: &str) -> Result<()>;
    /// 创建一个数据模型的字段
    async fn create_field(
        &self,
        account_id: &str,
        schema_id: &str,
        field: &io_schema::CreateField,
    ) -> Result<FieldModel>;
    /// 查询一个数据模型的字段
    async fn get_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
    ) -> Result<FieldModel>;
    /// 更新数据模型的字段信息
    async fn update_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
        req: &io_schema::UpdateField,
    ) -> Result<FieldModel>;
    /// 删除数据模型的字段
    async fn delete_field(&self, account_id: &str, schema_id: &str, identifier: &str)
        -> Result<()>;
}
