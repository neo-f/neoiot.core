use chrono::{DateTime, Local};
use entity::{fields, prelude::*};
use poem_openapi::{
    types::{Email, Password},
    Object,
};

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct AccountResp {
    pub id: String,
    /// 账户唯一邮箱
    pub email: Email,
    /// 账户名称
    pub name: String,
    /// 是否超级用户
    pub is_superuser: bool,
    /// 上次登录时间
    pub last_login_at: Option<DateTime<Local>>,
    /// 账户创建时间
    pub created_at: DateTime<Local>,
}

impl From<AccountModel> for AccountResp {
    fn from(obj: AccountModel) -> Self {
        Self {
            id: obj.id,
            email: Email(obj.email),
            name: obj.name,
            is_superuser: obj.is_superuser,
            last_login_at: obj.last_login_at.map(|v| v.into()),
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct AccountCreateReq {
    /// 账户唯一邮箱
    pub email: Email,
    /// 账户名称
    #[oai(validator(min_length = 3, max_length = 64))]
    pub name: String,
    /// 账户密码
    #[oai(validator(min_length = 8))]
    pub password: Password,
    pub is_super: bool,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct AccountUpdateReq {
    /// 账户唯一邮箱
    pub email: Option<Email>,
    /// 账户名称
    pub name: Option<String>,
    /// 账户密码
    pub password: Option<Password>,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct AccountListResp {
    /// 数据列表
    pub results: Vec<AccountResp>,
    /// 总数
    pub total: usize,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct LoginResp {
    pub token: String,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceConnectionResp {
    pub id: String,
    /// 所属设备原因
    pub device_id: String,
    /// 连接状态
    pub connected: bool,
    /// client_id
    pub client_id: String,
    /// 连接节点
    pub node: String,
    pub keep_alive: String,
    /// IP地址`
    pub ip_address: String,
    /// 协议版本
    pub proto_ver: i64,
    /// 连接时间
    pub connected_at: DateTimeWithTimeZone,
    /// 断开连接的时间
    pub disconnected_at: DateTimeWithTimeZone,
    /// 断开连接原因
    pub disconnected_reason: String,
}
impl From<DeviceConnectionModel> for DeviceConnectionResp {
    fn from(model: DeviceConnectionModel) -> Self {
        Self {
            id: model.id,
            device_id: model.device_id,
            connected: model.connected,
            client_id: model.client_id,
            node: model.node,
            keep_alive: model.keep_alive,
            ip_address: model.ip_address,
            proto_ver: model.proto_ver,
            connected_at: model.connected_at,
            disconnected_at: model.disconnected_at,
            disconnected_reason: model.disconnected_reason,
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceConnectionsListResp {
    total: usize,
    results: Vec<DeviceConnectionResp>,
}
impl From<(Vec<DeviceConnectionModel>, usize)> for DeviceConnectionsListResp {
    fn from(tuple: (Vec<DeviceConnectionModel>, usize)) -> Self {
        let (models, total) = tuple;
        Self {
            total,
            results: models.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct DeviceModelWithRelated {
    pub device: DeviceModel,
    pub labels: Vec<LabelModel>,
    pub schema: SchemaModel,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceResp {
    /// 设备ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 设备标签列表
    pub labels: Vec<LabelResp>,
    /// 数据模型
    pub schema: SchemaResp,
    /// 设备是否激活
    pub is_active: bool,
    /// 设备是否在线
    pub is_online: bool,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
}

impl From<DeviceModelWithRelated> for DeviceResp {
    fn from(obj: DeviceModelWithRelated) -> Self {
        DeviceResp {
            id: obj.device.id,
            name: obj.device.name,
            labels: obj.labels.into_iter().map(|x| x.into()).collect(),
            schema: obj.schema.into(),
            is_active: obj.device.is_active,
            is_online: obj.device.is_online,
            created_at: obj.device.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceSimpleResp {
    /// 设备ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 数据模型ID
    pub schema_id: String,
    /// 设备是否激活
    pub is_active: bool,
    /// 设备是否在线
    pub is_online: bool,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
}
impl From<DeviceModel> for DeviceSimpleResp {
    fn from(obj: DeviceModel) -> Self {
        DeviceSimpleResp {
            id: obj.id,
            name: obj.name,
            schema_id: obj.schema_id.to_string(),
            is_active: obj.is_active,
            is_online: obj.is_online,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceCreateReq {
    /// 设备名称
    pub name: String,
    /// 数据模型ID
    pub schema_id: String,
    /// 标签列表
    pub labels: Vec<String>,
    /// 设备MQTT连接密码
    pub mqtt_password: String,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceUpdateReq {
    /// 设备名称
    pub name: Option<String>,
    /// 设备激活状态ID
    pub is_active: Option<bool>,
    /// 设备标签
    pub labels: Option<Vec<String>>,
    /// 数据模型
    pub schema_id: Option<String>,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct DeviceListResp {
    /// 数据列表
    pub results: Vec<DeviceSimpleResp>,
    /// 总数
    pub total: usize,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct LabelResp {
    /// 设备ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
}

impl From<LabelModel> for LabelResp {
    fn from(obj: LabelModel) -> Self {
        LabelResp {
            id: obj.id,
            name: obj.name,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct SchemaResp {
    pub id: String,
    /// 数据模型名称
    pub name: String,
    /// 数据模型创建时间
    pub created_at: DateTime<Local>,
}

impl From<SchemaModel> for SchemaResp {
    fn from(obj: SchemaModel) -> Self {
        Self {
            id: obj.id,
            name: obj.name,
            created_at: obj.created_at.into(),
        }
    }
}

pub struct SchemaModelWithRelated {
    pub schema: SchemaModel,
    pub field: Vec<FieldModel>,
}
pub struct SchemaDetailResp {
    pub id: String,
    /// 数据模型名称
    pub name: String,
    /// 数据模型创建时间
    pub created_at: DateTime<Local>,
    /// 属性
    pub field: Vec<FieldResp>,
}

impl From<SchemaModelWithRelated> for SchemaDetailResp {
    fn from(obj: SchemaModelWithRelated) -> Self {
        Self {
            id: obj.schema.id,
            name: obj.schema.name,
            created_at: obj.schema.created_at.into(),
            field: obj.field.into_iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct SchemaCreateReq {
    /// 数据模型名称
    #[oai(validator(min_length = 3, max_length = 64))]
    pub name: String,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct SchemaUpdateReq {
    /// 账户名称
    pub name: Option<String>,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct SchemaListResp {
    /// 数据列表
    pub results: Vec<SchemaResp>,
    /// 总数
    pub total: usize,
}

#[derive(Debug, Object, Clone, PartialEq)]
pub struct FieldResp {
    pub id: String,
    /// 属性唯一标识符
    pub identifier: String,
    // 所属数据模型ID
    pub schema_id: String,
    // 数据类型
    pub data_type: fields::DataType,
    // 备注信息
    pub comment: String,
    // 单位
    pub unit: String,
    /// 属性创建时间
    pub created_at: DateTime<Local>,
}
impl From<FieldModel> for FieldResp {
    fn from(obj: FieldModel) -> Self {
        Self {
            id: obj.id.clone(),
            identifier: obj.identifier,
            schema_id: obj.schema_id,
            data_type: obj.data_type,
            comment: obj.comment,
            unit: obj.unit,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, Clone, PartialEq)]
pub struct FieldCreateReq {
    /// 属性唯一标识符
    pub identifier: String,
    // 数据类型
    pub data_type: fields::DataType,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
}

#[derive(Debug, Object, Clone, PartialEq)]
pub struct FieldUpdateReq {
    /// 属性唯一标识符
    pub identifier: Option<String>,
    // 数据类型
    pub data_type: Option<fields::DataType>,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
}
