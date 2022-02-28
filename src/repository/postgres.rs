use std::time::SystemTime;
use std::{collections::HashSet, time::Duration};

use crate::topics::ACLRules;
use crate::{
    errors::NeoiotError,
    errors::Result,
    oai_schema::{
        CreateAccount, CreateDevice, CreateField, CreateSchema, DeviceModelWithRelated,
        SchemaModelWithRelated, SendCommandToDevice, UpdateAccount, UpdateDevice, UpdateField,
        UpdateSchema,
    },
    topics::{self, Message, Topics},
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::Local;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use entity::{accounts, device_connections, devices, fields, labels, schemas};
use entity::{prelude::*, sea_orm::ConnectOptions};
use poem::async_trait;
use poem_openapi::types::{Email, Password};
use rand_core::OsRng;
use serde_json::json;

use super::Repository;

#[derive(Clone)]
pub struct PostgresRepository {
    pub conn: DatabaseConnection,
}
const ADMIN_EMAIL: &str = "admin@neoiot.com";
const ADMIN_NAME: &str = "admin";
const ADMIN_PASSWORD: &str = "123123";

impl PostgresRepository {
    pub async fn new(dsn: impl Into<String>) -> Self {
        let mut opt = ConnectOptions::new(dsn.into());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8));

        let conn = Database::connect(opt).await.unwrap();
        Self { conn }
    }
    // 初始化管理员账号
    pub async fn initial_admin(&self) {
        let check_admin = self.get_account_by_email(ADMIN_EMAIL).await;
        if check_admin.is_err() {
            let req = CreateAccount {
                email: Email(ADMIN_EMAIL.into()),
                name: ADMIN_NAME.into(),
                password: Password(ADMIN_PASSWORD.into()),
                is_super: true,
            };
            self.create_account(&req).await.unwrap();
        };
    }
}

#[async_trait]
impl super::Repository for PostgresRepository {
    async fn create_account(&self, req: &CreateAccount) -> Result<AccountModel> {
        let new_account = AccountActiveModel {
            id: Set(xid::new().to_string()),
            email: Set(req.email.to_string()),
            name: Set(req.name.clone()),
            password: Set(hash_password(&req.password)),
            is_superuser: Set(req.is_super),
            ..Default::default()
        };
        let account = new_account.insert(&self.conn).await?;
        Ok(account)
    }
    async fn get_account(&self, account_id: &str) -> Result<AccountModel> {
        let obj = AccountEntity::find_by_id(account_id.to_string())
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("account".to_string()))?;
        Ok(obj)
    }
    async fn get_account_by_email(&self, email: &str) -> Result<AccountModel> {
        let obj = AccountEntity::find()
            .filter(accounts::Column::Email.eq(email))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("account".to_string()))?;
        Ok(obj)
    }
    async fn after_account_logined(&self, email: &str) -> Result<()> {
        let account = self.get_account_by_email(email).await?;
        let mut account: AccountActiveModel = account.into();
        account.last_login_at = Set(Some(Local::now().into()));
        account.update(&self.conn).await?;
        Ok(())
    }

    async fn list_account(
        &self,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<AccountModel>, usize)> {
        let mut stmt = AccountEntity::find();
        if let Some(q) = q {
            stmt = stmt.filter(accounts::Column::Email.starts_with(&q));
        }
        if let Some(id_in) = id_in {
            stmt = stmt.filter(accounts::Column::Id.is_in(id_in));
        }
        let stmt = stmt
            .order_by_asc(accounts::Column::Id)
            .paginate(&self.conn, page_size);
        let objects = stmt.fetch_page(page - 1).await?;
        let total = stmt.num_items().await?;
        Ok((objects, total))
    }

    async fn update_account(&self, id: &str, req: &UpdateAccount) -> Result<AccountModel> {
        let obj = self.get_account(id).await?;
        let mut obj: AccountActiveModel = obj.into();
        if let Some(email) = &req.email {
            obj.email = Set(email.to_string());
        }
        if let Some(name) = &req.name {
            obj.name = Set(name.clone());
        }
        if let Some(password) = &req.password {
            obj.password = Set(hash_password(password));
        }
        let account = obj.update(&self.conn).await?;
        Ok(account)
    }

    async fn delete_account(&self, account_id: &str) -> Result<()> {
        let account = self.get_account(account_id).await?;
        account.delete(&self.conn).await?;
        Ok(())
    }

    async fn get_device(&self, account_id: &str, device_id: &str) -> Result<DeviceModel> {
        let device = DeviceEntity::find()
            .filter(devices::Column::Id.eq(device_id))
            .filter(devices::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("device".to_string()))?;
        Ok(device)
    }

    async fn get_device_with_labels(
        &self,
        account_id: &str,
        device_id: &str,
    ) -> Result<DeviceModelWithRelated> {
        let (device, schema) = DeviceEntity::find()
            .find_with_related(SchemaEntity)
            .filter(devices::Column::Id.eq(device_id))
            .filter(devices::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("device".to_string()))?;
        let labels = device.find_related(LabelEntity).all(&self.conn).await?;
        Ok(DeviceModelWithRelated {
            device,
            labels,
            schema: schema.unwrap(),
        })
    }

    async fn list_device(
        &self,
        account_id: Option<&str>,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        labels_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<DeviceModel>, usize)> {
        let mut stmt = DeviceEntity::find();
        if let Some(id_in) = id_in {
            stmt = stmt.filter(devices::Column::Id.is_in(id_in));
        }
        if let Some(q) = q {
            stmt = stmt.filter(devices::Column::Name.starts_with(&q));
        }
        if let Some(account_id) = account_id {
            stmt = stmt.filter(devices::Column::AccountId.eq(account_id));
        }
        if let Some(labels) = labels_in {
            stmt = stmt
                .right_join(LabelEntity)
                .filter(labels::Column::Name.is_in(labels));
        }
        let stmt = stmt
            .order_by_asc(devices::Column::Id)
            .paginate(&self.conn, page_size);
        let devices = stmt.fetch_page(page - 1).await?;
        let total = stmt.num_items().await?;

        Ok((devices, total))
    }

    async fn update_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &UpdateDevice,
    ) -> Result<DeviceModelWithRelated> {
        let device_with_labels = self.get_device_with_labels(account_id, device_id).await?;
        let mut device: devices::ActiveModel = device_with_labels.device.into();
        //1. change device tags
        if let Some(new_labels) = &req.labels {
            let old_labels = device_with_labels
                .labels
                .iter()
                .map(|l| l.name.clone())
                .collect::<HashSet<_>>();
            let new_labels = new_labels.iter().cloned().collect::<HashSet<_>>();
            if old_labels != new_labels {
                LabelEntity::delete_many()
                    .filter(labels::Column::DeviceId.eq(device_id))
                    .exec(&self.conn)
                    .await?;
                LabelEntity::insert_many(new_labels.into_iter().map(|l| labels::ActiveModel {
                    id: Set(xid::new().to_string()),
                    device_id: Set(device_id.to_string()),
                    name: Set(l),
                    ..Default::default()
                }))
                .exec(&self.conn)
                .await?;
                device.label_version = Set(SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64);
            }
        }
        if let Some(name) = &req.name {
            device.name = Set(name.clone());
        }
        if let Some(is_active) = &req.is_active {
            device.is_active = Set(*is_active);
        }
        if let Some(schema_id) = &req.schema_id {
            device.schema_id = Set(schema_id.clone());
        }
        device.update(&self.conn).await?;
        self.get_device_with_labels(account_id, device_id).await
    }

    async fn delete_device(&self, account_id: &str, device_id: &str) -> Result<()> {
        let device = self.get_device(account_id, device_id).await?;
        device.delete(&self.conn).await?;
        Ok(())
    }

    async fn create_device(
        &self,
        account_id: &str,
        req: &CreateDevice,
    ) -> Result<DeviceModelWithRelated> {
        let device_id = xid::new().to_string();
        let acl = ACLRules::new(account_id.to_string(), device_id.clone());
        let new_device = devices::ActiveModel {
            id: Set(device_id.clone()),
            account_id: Set(account_id.to_string()),
            schema_id: Set(req.schema_id.clone()),
            name: Set(req.name.clone()),
            label_version: Set(0),
            is_active: Set(true),
            is_online: Set(false),
            mqtt_username: Set(format!("{}/{}", &device_id, account_id)),
            mqtt_password: Set(req.mqtt_password.clone()),
            acl_pubs: Set(json!([
                acl.pub_d2d(),
                acl.pub_d2s(),
                acl.pub_s2dr(),
                acl.pub_metrics(),
            ])),
            acl_subs: Set(json!([acl.sub_s2d(), acl.sub_s2ds(), acl.sub_d2d()])),
            is_super_device: Set(false),
            ..Default::default()
        };

        let new_labels = req
            .labels
            .clone()
            .into_iter()
            .map(|l| labels::ActiveModel {
                id: Set(xid::new().to_string()),
                device_id: Set(device_id.clone()),
                name: Set(l),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        new_device.insert(&self.conn).await?;
        LabelEntity::insert_many(new_labels)
            .exec(&self.conn)
            .await?;
        self.get_device_with_labels(account_id, &device_id).await
    }

    async fn list_device_connections(
        &self,
        account_id: &str,
        device_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<DeviceConnectionModel>, usize)> {
        self.get_device(account_id, device_id).await?;
        let paginator = DeviceConnectionEntity::find()
            .filter(device_connections::Column::DeviceId.eq(device_id))
            .order_by_asc(device_connections::Column::Id)
            .paginate(&self.conn, page_size);
        let connections = paginator.fetch_page(page - 1).await?;
        let total = paginator.num_items().await?;

        Ok((connections, total))
    }
    async fn send_command_to_device(
        &self,
        account_id: &str,
        device_id: &str,
        req: &SendCommandToDevice,
    ) -> Result<String> {
        let _device = self.get_device(account_id, device_id).await?;
        let command =
            topics::ServerToDevice::new(account_id, device_id, &req.command, req.is_sync, req.ttl);
        let message_id = command.message_id.clone();
        Message::new(Topics::S2D(command), req.payload.clone())
            .publish(req.qos)
            .await?;
        Ok(message_id)
    }
    async fn create_schema(&self, account_id: &str, schema: &CreateSchema) -> Result<SchemaModel> {
        let new_schema = SchemaActiveModel {
            id: Set(xid::new().to_string()),
            account_id: Set(account_id.to_string()),
            name: Set(schema.name.clone()),
            ..Default::default()
        };
        let new_schema = new_schema.insert(&self.conn).await?;
        Ok(new_schema)
    }

    async fn get_schema_with_related(
        &self,
        account_id: &str,
        schema_id: &str,
    ) -> Result<SchemaModelWithRelated> {
        let schema = SchemaEntity::find()
            .filter(schemas::Column::Id.eq(schema_id))
            .filter(schemas::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("schema".to_string()))?;
        let fields = schema.find_related(FieldEntity).all(&self.conn).await?;
        Ok(SchemaModelWithRelated { schema, fields })
    }
    async fn get_schema(&self, account_id: &str, schema_id: &str) -> Result<SchemaModel> {
        let schema = SchemaEntity::find()
            .filter(schemas::Column::Id.eq(schema_id))
            .filter(schemas::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("schema".to_string()))?;
        Ok(schema)
    }

    async fn list_schema(
        &self,
        account_id: &str,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<SchemaModel>, usize)> {
        let mut stmt = SchemaEntity::find().filter(schemas::Column::AccountId.eq(account_id));
        if let Some(id_in) = id_in {
            stmt = stmt.filter(schemas::Column::Id.is_in(id_in));
        }
        if let Some(q) = q {
            stmt = stmt.filter(schemas::Column::Name.starts_with(&q));
        }
        let stmt = stmt
            .order_by_asc(schemas::Column::Id)
            .paginate(&self.conn, page_size);
        let schemas = stmt.fetch_page(page - 1).await?;
        let total = stmt.num_items().await?;

        Ok((schemas, total))
    }
    async fn update_schema(
        &self,
        account_id: &str,
        schema_id: &str,
        req: &UpdateSchema,
    ) -> Result<SchemaModel> {
        let schema = self.get_schema(account_id, schema_id).await?;
        let mut schema: schemas::ActiveModel = schema.into();
        if let Some(name) = &req.name {
            schema.name = Set(name.clone());
        }
        let schema = schema.update(&self.conn).await?;
        Ok(schema)
    }

    async fn delete_schema(&self, account_id: &str, schema_id: &str) -> Result<()> {
        let schema = self.get_schema(account_id, schema_id).await?;
        schema.delete(&self.conn).await?;
        Ok(())
    }
    async fn get_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
    ) -> Result<FieldModel> {
        let field = FieldEntity::find()
            .left_join(SchemaEntity)
            .filter(schemas::Column::AccountId.eq(account_id))
            .filter(fields::Column::SchemaId.eq(schema_id))
            .filter(fields::Column::Identifier.eq(identifier))
            .one(&self.conn)
            .await?
            .ok_or_else(|| NeoiotError::ObjectNotFound("field".to_string()))?;
        Ok(field)
    }
    async fn create_field(
        &self,
        account_id: &str,
        schema_id: &str,
        field: &CreateField,
    ) -> Result<FieldModel> {
        self.get_schema(account_id, schema_id).await?;
        let new_field = FieldActiveModel {
            id: Set(xid::new().to_string()),
            schema_id: Set(schema_id.to_string()),
            identifier: Set(field.identifier.clone()),
            data_type: Set(field.data_type.clone()),
            ..Default::default()
        };
        let field = new_field.insert(&self.conn).await?;
        Ok(field)
    }
    async fn update_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
        req: &UpdateField,
    ) -> Result<FieldModel> {
        let field = self.get_field(account_id, schema_id, identifier).await?;
        let mut field: FieldActiveModel = field.into();
        if let Some(identifier) = &req.identifier {
            field.identifier = Set(identifier.clone());
        };
        if let Some(data_type) = &req.data_type {
            field.data_type = Set(data_type.clone());
        };
        if let Some(comment) = &req.comment {
            field.comment = Set(comment.clone());
        };
        if let Some(unit) = &req.unit {
            field.unit = Set(unit.clone());
        };
        let field = field.update(&self.conn).await?;
        Ok(field)
    }

    async fn delete_field(
        &self,
        account_id: &str,
        schema_id: &str,
        identifier: &str,
    ) -> Result<()> {
        let field = self.get_field(account_id, schema_id, identifier).await?;
        field.delete(&self.conn).await?;
        Ok(())
    }
}

fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
