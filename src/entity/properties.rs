//! SeaORM Entity. Generated by sea-orm-codegen 0.4.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "properties")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub mapping_id: String,
    pub identifier: String,
    pub data_type: DataType,
    pub comment: String,
    pub unit: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mappings::Entity",
        from = "Column::MappingId",
        to = "super::mappings::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Mappings,
}

impl Related<super::mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Mappings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Enum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "data_type")]
pub enum DataType {
    #[sea_orm(string_value = "string")]
    String,
    #[sea_orm(string_value = "number")]
    Number,
    #[sea_orm(string_value = "integer")]
    Integer,
    #[sea_orm(string_value = "boolean")]
    Boolean,
    #[sea_orm(string_value = "time")]
    Time,
}

use chrono::{DateTime, Local};
use poem_openapi::{Enum, Object};

#[derive(Debug, Object, Clone, PartialEq)]
pub struct PropertyResp {
    pub id: String,
    /// 属性唯一标识符
    pub identifier: String,
    // 所属映射集ID
    pub mapping_id: String,
    // 数据类型
    pub data_type: DataType,
    // 备注信息
    pub comment: String,
    // 单位
    pub unit: String,
    /// 属性创建时间
    pub created_at: DateTime<Local>,
}
impl From<Model> for PropertyResp {
    fn from(obj: Model) -> Self {
        Self {
            id: obj.id.clone(),
            identifier: obj.identifier,
            mapping_id: obj.mapping_id,
            data_type: obj.data_type,
            comment: obj.comment,
            unit: obj.unit,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, PartialEq)]
pub struct PropertyCreateReq {
    /// 属性唯一标识符
    pub identifier: String,
    // 数据类型
    pub data_type: DataType,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
}

#[derive(Debug, Object, Clone, PartialEq)]
pub struct PropertyUpdateReq {
    /// 属性唯一标识符
    pub identifier: Option<String>,
    // 数据类型
    pub data_type: Option<DataType>,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
}
