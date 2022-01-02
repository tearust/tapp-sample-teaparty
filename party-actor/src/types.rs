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

    pub msg_id: String,
    pub time: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageRequest {
    pub tapp_id: u64,
    pub channel: String,

    pub msg_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryBalanceRequest {
    pub tapp_id: u64,
    pub address: String,
    pub uuid: String,
    pub auth: AuthKey,
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