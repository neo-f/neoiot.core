use anyhow::Result;
use entity::prelude::*;
use poem::async_trait;

mod postgres;
pub use postgres::PostgresRepository;

use crate::io_schema::{self, SchemaModelWithRelated};

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn create_account(&self, req: &io_schema::CreateAccount) -> Result<AccountModel>;
    async fn get_account(&self, account_id: &str) -> Result<AccountModel>;
    async fn get_account_by_email(&self, email: &str) -> Result<AccountModel>;
    async fn after_account_logined(&self, email: &str) -> Result<()>;
    async fn list_account(
        &self,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<AccountModel>, usize)>;
    async fn update_account(
        &self,
        account_id: &str,
        req: &io_schema::UpdateAccount,
    ) -> Result<AccountModel>;
    async fn delete_account(&self, account_id: &str) -> Result<()>;

    async fn get_device(&self, account_id: &str, device_id: &str) -> Result<DeviceModel>;
    async fn get_device_with_labels(
        &self,
        account_id: &str,
        device_id: &str,
    ) -> Result<io_schema::DeviceModelWithRelated>;
    async fn list_device(
        &self,
        account_id: Option<&str>,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        labels_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<DeviceModel>, usize)>;
    async fn update_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &io_schema::UpdateDevice,
    ) -> Result<io_schema::DeviceModelWithRelated>;
    async fn delete_device(&self, account_id: &str, device_id: &str) -> Result<()>;
    async fn create_device(
        &self,
        account_id: &str,
        req: &io_schema::CreateDevice,
    ) -> Result<io_schema::DeviceModelWithRelated>;

    async fn create_schema(
        &self,
        account_id: &str,
        schema: &io_schema::CreateSchema,
    ) -> Result<SchemaModel>;
    async fn get_schema(&self, account_id: &str, schema_id: &str)
        -> Result<SchemaModelWithRelated>;
    async fn list_schema(
        &self,
        account_id: &str,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<SchemaModel>, usize)>;
    async fn update_schema(
        &self,
        account_id: &str,
        schema_id: &str,
        req: &io_schema::UpdateSchema,
    ) -> Result<SchemaModel>;
    async fn delete_schema(&self, account_id: &str, schema_id: &str) -> Result<()>;
    async fn list_device_connections(
        &self,
        account_id: &str,
        device_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<DeviceConnectionModel>, usize)>;
    async fn create_field(
        &self,
        account_id: &str,
        schema_id: &str,
        field: &io_schema::CreateField,
    ) -> Result<FieldModel>;
    async fn update_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
        req: &io_schema::UpdateField,
    ) -> Result<FieldModel>;
}
