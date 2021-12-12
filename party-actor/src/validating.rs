use crate::types::*;
use crate::BINDING_NAME;
use tea_actor_utility::{
	actor_crypto::{
		aes_decrypt, aes_encrypt, generate_aes_key, generate_rsa_keypair, public_key_from_ss58,
		public_key_to_ss58, rsa_decrypt, sign, verify,
	},
	actor_enclave::{get_my_ephemeral_id, get_my_ephemeral_key},
	actor_kvp,
};

const COMMUNICATION_AES_KEY_PREFIX: &str = "tapp_bbs_aes";
const COMMUNICATION_RSA_KEY_PREFIX: &str = "tapp_bbs_rsa";

pub(crate) fn prepare_login(_uuid: &str, request: &PrepareLoginRequest) -> anyhow::Result<Vec<u8>> {
	let sr25519_pubkey = public_key_from_ss58(&request.address)?;

	if !verify(
		"sr25519".into(),
		sr25519_pubkey.clone(),
		base64::decode(&request.data)?,
		base64::decode(&request.signature)?,
	)? {
		return Err(anyhow::anyhow!("Validate account signature failed"));
	}

	let aes_key = communication_aes_key(&request.address);
	if actor_kvp::exists(BINDING_NAME, &aes_key).map_err(|e| anyhow::anyhow!("actor_kvp exists aes_key error:{}", e))? {
		actor_kvp::del(BINDING_NAME, &aes_key).map_err(|e| anyhow::anyhow!("actor_kvp del aes_key error: {}", e))?;
	}

	let (rsa_pub, rsa_pri) = generate_rsa_keypair(512)?;
	actor_kvp::set(
		BINDING_NAME,
		&communication_rsa_key(&request.address),
		&rsa_pri,
		6000,
	)
	.map_err(|e| anyhow::anyhow!("actor_kvp set ras_pri error:{}", e))?;

	let sign_data = sr25519_pubkey;
	let signature = sign("ed25519".into(), get_my_ephemeral_key()?, sign_data.clone())?;

	let response = PrepareLoginResponse {
		ephemeral_public_key: base64::encode(get_my_ephemeral_id()?),
		rsa_public_key: base64::encode(rsa_pub),
		sign_data: base64::encode(sign_data),
		signature: base64::encode(signature),
	};
	Ok(serde_json::to_string(&response)?.into_bytes())
}

pub(crate) fn login(_uuid: &str, request: &LoginRequest) -> anyhow::Result<Vec<u8>> {
	let rsa_key: String = actor_kvp::get(BINDING_NAME, &communication_rsa_key(&request.address))?
		.ok_or(anyhow::anyhow!("failed to get rsa key"))?;
	let aes_key = rsa_decrypt(rsa_key, base64::decode(&request.encrypted_aes_key)?)?;
	actor_kvp::set_forever(
		BINDING_NAME,
		&communication_aes_key(&request.address),
		&aes_key,
	)?;

	Ok(serde_json::to_string(&LoginResponse { success: true })?.into_bytes())
}

pub(crate) fn logout(_uuid: &str, request: &LogoutRequest) -> anyhow::Result<Vec<u8>> {
	actor_kvp::del(BINDING_NAME, &communication_aes_key(&request.address))
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	Ok(serde_json::to_string(&LogoutResponse { success: true })?.into_bytes())
}

pub(crate) fn is_user_logged_in(address: &str) -> anyhow::Result<bool> {
	let exist = actor_kvp::exists(BINDING_NAME, &communication_aes_key(address))
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	Ok(exist)
}

pub(crate) fn decrypt_message(message: &str, address: &str) -> anyhow::Result<String> {
	let aes_key = actor_kvp::get(BINDING_NAME, &communication_aes_key(address))?;
	match aes_key {
		Some(key) => {
			let result = aes_decrypt(key, base64::decode(message)?)?;
			Ok(String::from_utf8(result)?)
		}
		None => Err(anyhow::anyhow!(
			"failed to get aes key, can not decrypt message"
		)),
	}
}

fn get_aes_key_from_appstore() -> anyhow::Result<Vec<u8>> {
	warn!("todo: get Aes key from app store.");

	let aes_key: Vec<u8> = vec![8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];
	// let aes_key = generate_aes_key()?;
	Ok(aes_key)
}

pub(crate) fn aes_encrypt_local(message: &str) -> anyhow::Result<String> {
	let aes_key = get_aes_key_from_appstore()?;

	// TODO will fail when data size less than 8. throw crypto provider handle_call error: BlockModeError
	// let data = b"hello12";
	// let encrypted_data = aes_encrypt(aes_key.clone(), data.to_vec())?;
	// info!("encrypt data: {}", base64::encode(encrypted_data.clone()));

	let mut data = message.as_bytes().to_vec();
	if data.len() < 8 {
		data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
	}

	let key = get_aes_key_from_appstore()?;
	let result = aes_encrypt(key, data)?;
	Ok(base64::encode(result))
}
pub(crate) fn aes_decrypt_local(encrypt_b64_message: &str) -> anyhow::Result<String> {
	let key = get_aes_key_from_appstore()?;
	let result = aes_decrypt(key, base64::decode(encrypt_b64_message)?)?;
	Ok(String::from_utf8(result)?)
}

pub(crate) fn communication_aes_key(key: &str) -> String {
	format!("{}-{}", COMMUNICATION_AES_KEY_PREFIX, key)
}

pub(crate) fn communication_rsa_key(key: &str) -> String {
	format!("{}-{}", COMMUNICATION_RSA_KEY_PREFIX, key)
}
