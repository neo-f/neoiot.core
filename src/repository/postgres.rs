use std::collections::HashSet;
use std::time::SystemTime;

use crate::entity::prelude::*;
use crate::entity::{accounts, device_connections, devices, labels, mappings, properties};
use anyhow::Result;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use poem::async_trait;
use poem::error::NotFoundError;
use rand_core::OsRng;
use sea_orm::{prelude::*, QueryOrder, Set};

pub struct PostgresRepository {
    pub conn: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl super::Repository for PostgresRepository {
    async fn create_account(&self, req: &accounts::AccountCreateReq) -> Result<accounts::Model> {
        let new_account = accounts::ActiveModel {
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
    async fn get_account(&self, account_id: &str) -> Result<accounts::Model> {
        let obj = Accounts::find_by_id(account_id.to_string())
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        Ok(obj)
    }
    async fn get_account_by_email(&self, email: &str) -> Result<accounts::Model> {
        let obj = Accounts::find()
            .filter(accounts::Column::Email.eq(email))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        Ok(obj)
    }

    async fn list_account(
        &self,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<accounts::Model>, usize)> {
        let mut stmt = Accounts::find();
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

    async fn update_account(
        &self,
        id: &str,
        req: &accounts::AccountUpdateReq,
    ) -> Result<accounts::Model> {
        let obj = self.get_account(id).await?;
        let mut obj: accounts::ActiveModel = obj.into();
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
    async fn get_device(&self, account_id: &str, device_id: &str) -> Result<devices::Model> {
        let device = Devices::find()
            .filter(devices::Column::Id.eq(device_id))
            .filter(devices::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        Ok(device)
    }

    async fn get_device_with_labels(
        &self,
        account_id: &str,
        device_id: &str,
    ) -> Result<devices::ModelWithRelated> {
        let (device, mapping) = Devices::find()
            .find_with_related(Mappings)
            .filter(devices::Column::Id.eq(device_id))
            .filter(devices::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        let labels = device.find_related(Labels).all(&self.conn).await?;
        Ok(devices::ModelWithRelated {
            device,
            labels,
            mapping: mapping.unwrap(),
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
    ) -> Result<(Vec<devices::Model>, usize)> {
        let mut stmt = Devices::find();
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
                .right_join(Labels)
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
        req: &devices::DeviceUpdateReq,
    ) -> Result<devices::ModelWithRelated> {
        let device_with_labels = self.get_device_with_labels(account_id, device_id).await?;
        if device_with_labels.device.account_id != account_id {
            return Err(NotFoundError.into());
        }
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
                Labels::delete_many()
                    .filter(labels::Column::DeviceId.eq(device_id))
                    .exec(&self.conn)
                    .await?;
                Labels::insert_many(new_labels.into_iter().map(|l| labels::ActiveModel {
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
        if let Some(mapping_id) = &req.mapping_id {
            device.mapping_id = Set(mapping_id.clone());
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
        req: &devices::DeviceCreateReq,
    ) -> Result<devices::ModelWithRelated> {
        let device_id = xid::new().to_string();
        let new_device = devices::ActiveModel {
            id: Set(device_id.clone()),
            account_id: Set(account_id.to_string()),
            mapping_id: Set(req.mapping_id.clone()),
            name: Set(req.name.clone()),
            label_version: Set(SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64),
            is_active: Set(true),
            is_online: Set(false),
            mqtt_username: Set(format!("{}/{}", &device_id, account_id)),
            mqtt_password: Set(req.mqtt_password.clone()),
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
        Labels::insert_many(new_labels).exec(&self.conn).await?;
        self.get_device_with_labels(account_id, &device_id).await
    }

    async fn create_mapping(
        &self,
        account_id: &str,
        mapping: &mappings::MappingCreateReq,
    ) -> Result<mappings::Model> {
        let new_mapping = mappings::ActiveModel {
            id: Set(xid::new().to_string()),
            account_id: Set(account_id.to_string()),
            name: Set(mapping.name.clone()),
            ..Default::default()
        };
        let new_mapping = new_mapping.insert(&self.conn).await?;
        Ok(new_mapping)
    }
    async fn get_mapping(&self, account_id: &str, mapping_id: &str) -> Result<mappings::Model> {
        let mapping = Mappings::find()
            .filter(mappings::Column::Id.eq(mapping_id))
            .filter(mappings::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        Ok(mapping)
    }
    async fn list_mapping(
        &self,
        account_id: &str,
        page: usize,
        page_size: usize,
        id_in: Option<Vec<String>>,
        q: Option<String>,
    ) -> Result<(Vec<mappings::Model>, usize)> {
        let mut stmt = Mappings::find().filter(mappings::Column::AccountId.eq(account_id));
        if let Some(id_in) = id_in {
            stmt = stmt.filter(mappings::Column::Id.is_in(id_in));
        }
        if let Some(q) = q {
            stmt = stmt.filter(mappings::Column::Name.starts_with(&q));
        }
        let stmt = stmt
            .order_by_asc(mappings::Column::Id)
            .paginate(&self.conn, page_size);
        let mappings = stmt.fetch_page(page - 1).await?;
        let total = stmt.num_items().await?;

        Ok((mappings, total))
    }

    async fn update_mapping(
        &self,
        account_id: &str,
        mapping_id: &str,
        req: &mappings::MappingUpdateReq,
    ) -> Result<mappings::Model> {
        let mapping = Mappings::find()
            .filter(mappings::Column::Id.eq(mapping_id))
            .filter(mappings::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        let mut mapping: mappings::ActiveModel = mapping.into();
        if let Some(name) = &req.name {
            mapping.name = Set(name.clone());
        }
        let mapping = mapping.update(&self.conn).await?;
        Ok(mapping)
    }
    async fn delete_mapping(&self, account_id: &str, mapping_id: &str) -> Result<()> {
        let mapping = Mappings::find()
            .filter(mappings::Column::Id.eq(mapping_id))
            .filter(mappings::Column::AccountId.eq(account_id))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        mapping.delete(&self.conn).await?;
        Ok(())
    }
    async fn list_device_connections(
        &self,
        account_id: &str,
        device_id: &str,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<device_connections::Model>, usize)> {
        self.get_device(account_id, device_id).await?;
        let paginator = DeviceConnections::find()
            .filter(device_connections::Column::DeviceId.eq(device_id))
            .order_by_asc(device_connections::Column::Id)
            .paginate(&self.conn, page_size);
        let connections = paginator.fetch_page(page - 1).await?;
        let total = paginator.num_items().await?;

        Ok((connections, total))
    }

    async fn create_property(
        &self,
        account_id: &str,
        mapping_id: &str,
        property: &properties::PropertyCreateReq,
    ) -> Result<properties::Model> {
        self.get_mapping(account_id, mapping_id).await?;
        let new_property = properties::ActiveModel {
            id: Set(xid::new().to_string()),
            mapping_id: Set(mapping_id.to_string()),
            identifier: Set(property.identifier.clone()),
            data_type: Set(property.data_type.clone()),
            ..Default::default()
        };
        let property = new_property.insert(&self.conn).await?;
        Ok(property)
    }
    async fn update_property(
        &self,
        account_id: &str,
        mapping_id: &str,
        identifier: &str,
        req: &properties::PropertyUpdateReq,
    ) -> Result<properties::Model> {
        let property = Properties::find()
            .left_join(Mappings)
            .filter(mappings::Column::AccountId.eq(account_id))
            .filter(properties::Column::MappingId.eq(mapping_id))
            .filter(properties::Column::Identifier.eq(identifier))
            .one(&self.conn)
            .await?
            .ok_or(NotFoundError)?;
        let mut property: properties::ActiveModel = property.into();
        if let Some(identifier) = &req.identifier {
            property.identifier = Set(identifier.clone());
        };
        if let Some(data_type) = &req.data_type {
            property.data_type = Set(data_type.clone());
        };
        if let Some(comment) = &req.comment {
            property.comment = Set(comment.clone());
        };
        if let Some(unit) = &req.unit {
            property.unit = Set(unit.clone());
        };
        let property = property.update(&self.conn).await?;
        Ok(property)
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
