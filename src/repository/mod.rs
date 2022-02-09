use crate::entity::{accounts, device_connections, devices, mappings, properties};
use anyhow::Result;
use poem::async_trait;

mod postgres;
pub use postgres::PostgresRepository;

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn create_account(&self, req: &accounts::AccountCreateReq) -> Result<accounts::Model>;
    async fn get_account(&self, account_id: &str) -> Result<accounts::Model>;
    async fn get_account_by_email(&self, email: &str) -> Result<accounts::Model>;
    async fn list_account(
        &self,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<accounts::Model>, usize)>;
    async fn update_account(
        &self,
        account_id: &str,
        req: &accounts::AccountUpdateReq,
    ) -> Result<accounts::Model>;
    async fn delete_account(&self, account_id: &str) -> Result<()>;

    async fn get_device(&self, account_id: &str, device_id: &str) -> Result<devices::Model>;
    async fn get_device_with_labels(
        &self,
        account_id: &str,
        device_id: &str,
    ) -> Result<devices::ModelWithRelated>;
    async fn list_device(
        &self,
        account_id: Option<&str>,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        labels_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<devices::Model>, usize)>;
    async fn update_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &devices::DeviceUpdateReq,
    ) -> Result<devices::ModelWithRelated>;
    async fn delete_device(&self, account_id: &str, device_id: &str) -> Result<()>;
    async fn create_device(
        &self,
        account_id: &str,
        req: &devices::DeviceCreateReq,
    ) -> Result<devices::ModelWithRelated>;

    async fn create_mapping(
        &self,
        account_id: &str,
        mapping: &mappings::MappingCreateReq,
    ) -> Result<mappings::Model>;
    async fn get_mapping(&self, account_id: &str, mapping_id: &str) -> Result<mappings::Model>;
    async fn list_mapping(
        &self,
        account_id: &str,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<mappings::Model>, usize)>;
    async fn update_mapping(
        &self,
        account_id: &str,
        mapping_id: &str,
        req: &mappings::MappingUpdateReq,
    ) -> Result<mappings::Model>;
    async fn delete_mapping(&self, account_id: &str, mapping_id: &str) -> Result<()>;
    async fn list_device_connections(
        &self,
        account_id: &str,
        device_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<device_connections::Model>, usize)>;
    async fn create_property(
        &self,
        account_id: &str,
        mapping_id: &str,
        property: &properties::PropertyCreateReq,
    ) -> Result<properties::Model>;
    async fn update_property(
        &self,
        account_id: &str,
        mapping_id: &str,
        identifier: &str,
        req: &properties::PropertyUpdateReq,
    ) -> Result<properties::Model>;
}
