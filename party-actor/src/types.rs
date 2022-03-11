use interface::{AuthKey, Balance};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMessageRequest {
	pub tapp_id: u64,
	pub channel: String,
	/// Base64 encoded
	pub encrypted_message: String,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
	pub ttl: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMessageResponse {
	pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMessageRequest {
	pub tapp_id: u64,
	pub channel: String,
	pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendMessageRequest {
	pub tapp_id: u64,
	pub channel: String,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
	pub msg_id: String,
	pub ttl: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageRequest {
	pub tapp_id: u64,
	pub channel: String,
	pub address: String,
	pub uuid: String,
	pub msg_id: String,
	pub auth_b64: String,
	pub is_tapp_owner: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryBalanceRequest {
	pub tapp_id: u64,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryResultWithUuid {
	pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageItem {
	pub tapp_id: u64,
	pub content: String,
	pub utc: u64,
	pub utc_expired: u64,
	pub id: String,
	pub sender: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMessageResponse {
	pub messages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareLoginRequest {
	pub tapp_id: u64,
	pub address: String,
	/// Base64 encoded
	pub data: String,
	/// Base64 encoded
	pub signature: String,
	pub uuid: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckLoginRequest {
	pub tapp_id: u64,
	pub address: String,
	pub auth_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareLoginResponse {
	/// Base64 encoded
	pub ephemeral_public_key: String,
	/// Base64 encoded
	pub rsa_public_key: String,
	/// Base64 encoded
	pub sign_data: String,
	/// Base64 encoded
	pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
	pub tapp_id: u64,
	pub address: String,
	/// Base64 encoded
	pub encrypted_aes_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
	pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
	pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutResponse {
	pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TappProfileRequest {
	pub tapp_id: u64,
	pub address: String,
	pub auth_b64: String,
	pub uuid: String,

	pub post_message_fee: Balance,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRequest {
	pub tapp_id: u64,
	pub address: String,
	pub auth_b64: String,
	pub uuid: String,
	pub amount: Balance,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHashRequest {
	pub uuid: String,
	pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTappAccountRequest {
	pub tapp_id: u64,
	pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTappStoreAccountRequest {
	pub uuid: String,
}

// notification
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationAddMessageRequest {
	pub tapp_id: u64,
	pub content_b64: String,
	pub from: String,
	pub to: String,
	pub uuid: String,
	pub auth_b64: String,
	pub from_tapp_id: u64,
	pub from_tapp_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationGetMessageRequest {
	pub tapp_id: u64,
	pub address: String,
	pub from: Option<String>,
	pub to: Option<String>,
	pub auth_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationMessageItem {
	pub tapp_id: u64,
	pub content: String,
	pub utc: u64,
	pub utc_expired: u64,
	pub id: String,
	pub sender: String,
	pub to: String,
	pub from_tapp_id: u64,
	pub from_tapp_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestForSqlRequest {
	pub tapp_id: u64,
	pub sql: String,
	pub is_txn: bool,
	pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestForComsumeDividend {
	pub tapp_id: u64,
	pub uuid: String,
}
