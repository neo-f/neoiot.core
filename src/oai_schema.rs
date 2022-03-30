use chrono::{DateTime, Local};
use entity::{fields, prelude::*, sea_orm::prelude::DateTimeWithTimeZone};
use poem_openapi::{
    payload::Json,
    types::{Email, MaybeUndefined, Password},
    ApiResponse, Enum, Object,
};

#[derive(Debug, Object, PartialEq)]
pub struct Account {
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

impl From<AccountModel> for Account {
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

#[derive(Debug, Object, PartialEq)]
pub struct CreateAccount {
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

#[derive(Debug, Object, PartialEq)]
pub struct UpdateAccount {
    /// 账户唯一邮箱
    pub email: Option<Email>,
    /// 账户名称
    pub name: Option<String>,
    /// 账户密码
    pub password: Option<Password>,
}

#[derive(Debug, Object, PartialEq)]
pub struct Accounts {
    /// 数据列表
    pub results: Vec<Account>,
    /// 总数
    pub total: usize,
}

#[derive(Debug, Object, PartialEq)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct DeviceConnection {
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
impl From<DeviceConnectionModel> for DeviceConnection {
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

#[derive(Debug, Object, PartialEq)]
pub struct DeviceConnections {
    total: usize,
    results: Vec<DeviceConnection>,
}
impl From<(Vec<DeviceConnectionModel>, usize)> for DeviceConnections {
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

#[derive(Debug, Object, PartialEq)]
pub struct DeviceWithLables {
    /// 设备ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 设备标签列表
    pub labels: Vec<String>,
    /// 数据模型
    pub schema: Schema,
    /// 设备是否激活
    pub is_active: bool,
    /// 设备是否在线
    pub is_online: bool,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
}

impl From<DeviceModelWithRelated> for DeviceWithLables {
    fn from(obj: DeviceModelWithRelated) -> Self {
        DeviceWithLables {
            id: obj.device.id,
            name: obj.device.name,
            labels: obj.labels.into_iter().map(|x| x.name).collect(),
            schema: obj.schema.into(),
            is_active: obj.device.is_active,
            is_online: obj.device.is_online,
            created_at: obj.device.created_at.into(),
        }
    }
}

#[derive(Debug, Object, PartialEq)]
pub struct Device {
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
impl From<DeviceModel> for Device {
    fn from(obj: DeviceModel) -> Self {
        Device {
            id: obj.id,
            name: obj.name,
            schema_id: obj.schema_id.to_string(),
            is_active: obj.is_active,
            is_online: obj.is_online,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, PartialEq)]
pub struct CreateDevice {
    /// 设备名称
    pub name: String,
    /// 数据模型ID
    pub schema_id: String,
    /// 标签列表
    pub label_ids: Vec<String>,
    /// 设备MQTT连接密码
    pub mqtt_password: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct UpdateDevice {
    /// 设备名称
    pub name: Option<String>,
    /// 设备激活状态ID
    pub is_active: Option<bool>,
    /// 设备标签
    pub label_ids: Option<Vec<String>>,
    /// 数据模型
    pub schema_id: Option<String>,
}

#[derive(Debug, Object, PartialEq)]
pub struct Devices {
    /// 数据列表
    pub results: Vec<Device>,
    /// 总数
    pub total: usize,
}

const fn default_qos() -> u8 {
    1
}
const fn default_async() -> bool {
    true
}
const fn default_codec() -> PayloadCodec {
    PayloadCodec::Plain
}
const fn default_sync_timeout() -> usize {
    10
}

#[derive(Debug, PartialEq, Enum)]
pub enum PayloadCodec {
    /// 不压缩
    Plain,
    /// Base64编码
    Base64,
}

#[derive(Debug, Object, PartialEq)]
pub struct SendCommandToDevice {
    /// 指令名称
    pub command: String,
    /// 编码类型
    #[oai(default = "default_codec")]
    pub codec: PayloadCodec,
    /// 负载信息
    pub payload: String,
    /// 是否需要同步
    /// - 同步模式下，发送指令后，接口会等待设备端指令执行结果并返回
    /// - 异步模式下，发送指令后，立即返回消息ID，不会返回指令执行结果
    #[oai(default = "default_async")]
    pub is_sync: bool,
    /// 同步模式时的最大等待时长（秒）
    #[oai(
        default = "default_sync_timeout",
        validator(maximum(value = "120"), minimum(value = "1"))
    )]
    pub sync_timeout: usize,
    /// 指令过期时间（秒）
    pub ttl: Option<usize>,
    /// 指令QOS
    #[oai(
        default = "default_qos",
        validator(maximum(value = "2"), minimum(value = "0"))
    )]
    pub qos: u8,
}

#[derive(Debug, Object, PartialEq)]
pub struct SendCommandToDeviceBatch {
    /// 指令名称
    pub command: String,
    /// 编码类型
    #[oai(default = "default_codec")]
    pub codec: PayloadCodec,
    /// 负载信息
    pub payload: String,
    /// 指令过期时间（秒）
    pub ttl: Option<usize>,
    /// 指令QOS
    #[oai(
        default = "default_qos",
        validator(maximum(value = "2"), minimum(value = "0"))
    )]
    pub qos: u8,
}

#[derive(Debug, Object, PartialEq)]
pub struct SyncCommandResponse {
    /// 设备响应
    pub response: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct AsyncCommandResponse {
    pub message_id: String,
}

#[derive(ApiResponse)]
pub enum CommandResponse {
    /// 同步模式下，设备端接收到指令后，会返回指令执行结果
    #[oai(status = "201")]
    Sync(Json<SyncCommandResponse>),
    /// 异步模式下，设备端接收到指令后，不会返回指令执行结果，只返回消息ID
    #[oai(status = "202")]
    Async(Json<AsyncCommandResponse>),
}
impl CommandResponse {
    pub fn new_sync(response: String) -> Self {
        CommandResponse::Sync(Json(SyncCommandResponse { response }))
    }
    pub fn new_async(message_id: String) -> Self {
        CommandResponse::Async(Json(AsyncCommandResponse { message_id }))
    }
}

#[derive(Debug, Object, PartialEq)]
pub struct Label {
    /// 设备ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
}

impl From<LabelModel> for Label {
    fn from(obj: LabelModel) -> Self {
        Label {
            id: obj.id,
            name: obj.name,
            created_at: obj.created_at.into(),
        }
    }
}
#[derive(Debug, Object, PartialEq)]
pub struct Labels {
    pub results: Vec<Label>,
}
#[derive(Debug, Object, PartialEq)]
pub struct UpdateLabel {
    pub name: String,
}
#[derive(Debug, Object, PartialEq)]
pub struct CreateLabel {
    pub name: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct Schema {
    pub id: String,
    /// 数据模型名称
    pub name: String,
    /// 数据模型创建时间
    pub created_at: DateTime<Local>,
}

impl From<SchemaModel> for Schema {
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
    pub fields: Vec<FieldModel>,
}

#[derive(Debug, Object, PartialEq)]
pub struct SchemaWithFields {
    pub id: String,
    /// 数据模型名称
    pub name: String,
    /// 数据模型创建时间
    pub created_at: DateTime<Local>,
    /// 字段
    pub fields: Vec<Field>,
}

impl From<SchemaModelWithRelated> for SchemaWithFields {
    fn from(obj: SchemaModelWithRelated) -> Self {
        Self {
            id: obj.schema.id,
            name: obj.schema.name,
            created_at: obj.schema.created_at.into(),
            fields: obj.fields.into_iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Debug, Object, PartialEq)]
pub struct CreateSchema {
    /// 数据模型名称
    #[oai(validator(min_length = 3, max_length = 64))]
    pub name: String,
}

#[derive(Debug, Object, PartialEq)]
pub struct UpdateSchema {
    /// 账户名称
    pub name: Option<String>,
}

#[derive(Debug, Object, PartialEq)]
pub struct Schemas {
    /// 数据列表
    pub results: Vec<Schema>,
    /// 总数
    pub total: usize,
}

#[derive(Debug, Object, PartialEq)]
pub struct Field {
    pub id: String,
    /// 字段唯一标识符
    pub identifier: String,
    // 数据类型
    pub data_type: fields::DataType,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
    /// 字段创建时间
    pub created_at: DateTime<Local>,
}
impl From<FieldModel> for Field {
    fn from(obj: FieldModel) -> Self {
        Self {
            id: obj.id.clone(),
            identifier: obj.identifier,
            data_type: obj.data_type,
            comment: obj.comment,
            unit: obj.unit,
            created_at: obj.created_at.into(),
        }
    }
}

#[derive(Debug, Object, PartialEq)]
pub struct CreateField {
    /// 字段唯一标识符
    pub identifier: String,
    // 数据类型
    pub data_type: fields::DataType,
    // 备注信息
    pub comment: Option<String>,
    // 单位
    pub unit: Option<String>,
}

#[derive(Debug, Object, PartialEq)]
pub struct UpdateField {
    /// 字段唯一标识符
    pub identifier: Option<String>,
    // 数据类型
    pub data_type: Option<fields::DataType>,
    // 备注信息
    pub comment: MaybeUndefined<String>,
    // 单位
    pub unit: MaybeUndefined<String>,
}
