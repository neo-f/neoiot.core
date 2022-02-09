//! SeaORM Entity. Generated by sea-orm-codegen 0.4.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "command_response_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub message_id: String,
    pub payload: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::command_request_logs::Entity",
        from = "Column::MessageId",
        to = "super::command_request_logs::Column::MessageId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    CommandRequestLogs,
}

impl Related<super::command_request_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CommandRequestLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
