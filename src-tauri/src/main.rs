#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey}, Signer, SigningKey, Verifier, VerifyingKey};
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tauri::path::BaseDirectory;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredDeviceIdentity {
    version: u8,
    device_id: String,
    public_key_pem: String,
    private_key_pem: String,
    created_at_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceIdentityPublic {
    version: u8,
    device_id: String,
    public_key: String,
    created_at_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignedPayload {
    device_id: String,
    public_key: String,
    signature: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChefVideoAssets {
    idle_path: String,
    busy_path: String,
}

fn now_ms() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("clock error: {err}"))?
        .as_millis() as u64)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn identity_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?;
    Ok(dir.join("identity").join("device.json"))
}

fn resolve_video_asset_path(app: &tauri::AppHandle, relative_path: &str) -> Result<PathBuf, String> {
    let resource_path = app
        .path()
        .resolve(relative_path, BaseDirectory::Resource)
        .map_err(|err| format!("failed to resolve resource path for {relative_path}: {err}"))?;
    if resource_path.exists() {
        return Ok(resource_path);
    }

    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(format!("video asset not found: {relative_path}"))
}

fn as_public(identity: &StoredDeviceIdentity) -> DeviceIdentityPublic {
    let public_key = VerifyingKey::from_public_key_pem(&identity.public_key_pem)
        .map(|key| URL_SAFE_NO_PAD.encode(key.to_bytes()))
        .unwrap_or_default();
    DeviceIdentityPublic {
        version: identity.version,
        device_id: identity.device_id.clone(),
        public_key,
        created_at_ms: identity.created_at_ms,
    }
}

fn read_identity(path: &PathBuf) -> Result<Option<StoredDeviceIdentity>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path).map_err(|err| format!("failed to read device identity: {err}"))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse device identity: {err}"))?;

    let Some(version) = parsed.get("version").and_then(|value| value.as_u64()) else {
        return Ok(None);
    };
    if version != 1 {
        return Ok(None);
    }

    let Some(_device_id) = parsed.get("deviceId").and_then(|value| value.as_str()) else {
        return Ok(None);
    };
    let Some(created_at_ms) = parsed.get("createdAtMs").and_then(|value| value.as_u64()) else {
        return Ok(None);
    };

    let Some(public_key_pem) = parsed.get("publicKeyPem").and_then(|value| value.as_str()) else {
        return Ok(None);
    };
    let Some(private_key_pem) = parsed.get("privateKeyPem").and_then(|value| value.as_str()) else {
        return Ok(None);
    };

    let derived_key = VerifyingKey::from_public_key_pem(public_key_pem)
        .map_err(|err| format!("failed to decode public key pem: {err}"))?;
    let derived_device_id = sha256_hex(&derived_key.to_bytes());

    Ok(Some(StoredDeviceIdentity {
        version: 1,
        device_id: derived_device_id,
        public_key_pem: public_key_pem.to_string(),
        private_key_pem: private_key_pem.to_string(),
        created_at_ms,
    }))
}

fn write_identity(path: &PathBuf, identity: &StoredDeviceIdentity) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create identity dir: {err}"))?;
    }

    let payload = serde_json::to_string_pretty(identity)
        .map_err(|err| format!("failed to serialize device identity: {err}"))?;
    fs::write(path, format!("{payload}\n")).map_err(|err| format!("failed to store device identity: {err}"))
}

fn load_or_create_identity(app: &tauri::AppHandle) -> Result<StoredDeviceIdentity, String> {
    let path = identity_file_path(app)?;
    if let Some(existing) = read_identity(&path)? {
        return Ok(existing);
    }

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let public_key_raw = verifying_key.to_bytes();
    let created_at_ms = now_ms()?;
    let identity = StoredDeviceIdentity {
        version: 1,
        device_id: sha256_hex(&public_key_raw),
        public_key_pem: verifying_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|err| format!("failed to export public key pem: {err}"))?,
        private_key_pem: signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|err| format!("failed to export private key pem: {err}"))?
            .to_string(),
        created_at_ms,
    };

    write_identity(&path, &identity)?;
    Ok(identity)
}

#[tauri::command]
fn load_or_create_device_identity(app: tauri::AppHandle) -> Result<DeviceIdentityPublic, String> {
    let identity = load_or_create_identity(&app)?;
    Ok(as_public(&identity))
}

#[tauri::command]
fn sign_device_payload(app: tauri::AppHandle, payload: String) -> Result<SignedPayload, String> {
    let identity = load_or_create_identity(&app)?;
    let signing_key = SigningKey::from_pkcs8_pem(&identity.private_key_pem)
        .map_err(|err| format!("failed to decode private key pem: {err}"))?;
    let verifying_key = signing_key.verifying_key();
    let signature = signing_key.sign(payload.as_bytes());

    verifying_key
        .verify(payload.as_bytes(), &signature)
        .map_err(|err| format!("signing self-verification failed: {err}"))?;

    let public_key = URL_SAFE_NO_PAD.encode(verifying_key.to_bytes());

    Ok(SignedPayload {
        device_id: identity.device_id,
        public_key,
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

#[tauri::command]
fn reset_device_identity(app: tauri::AppHandle) -> Result<(), String> {
    let path = identity_file_path(&app)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|err| format!("failed to remove device identity: {err}"))?;
    }
    Ok(())
}

#[tauri::command]
fn resolve_chef_video_assets(app: tauri::AppHandle) -> Result<ChefVideoAssets, String> {
    let idle_path = resolve_video_asset_path(&app, "videos/sharpening-web.mp4")
        .map_err(|err| format!("failed to resolve idle chef video: {err}"))?;
    let busy_path = resolve_video_asset_path(&app, "videos/cooking-web.mp4")
        .map_err(|err| format!("failed to resolve busy chef video: {err}"))?;

    Ok(ChefVideoAssets {
        idle_path: idle_path.to_string_lossy().into_owned(),
        busy_path: busy_path.to_string_lossy().into_owned(),
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_or_create_device_identity,
            sign_device_payload,
            reset_device_identity,
            resolve_chef_video_assets
        ])
        .setup(|app| {
            let window = app.get_webview_window("main")
                .expect("failed to get main window");

            if let Some(monitor) = window.current_monitor().ok().flatten()
                .or_else(|| window.primary_monitor().ok().flatten())
                .or_else(|| window.available_monitors().ok()
                    .and_then(|monitors| monitors.into_iter().next()))
            {
                let size = monitor.size();
                let _ = window.set_size(tauri::PhysicalSize::new(size.width, size.height));
                let _ = window.set_position(tauri::PhysicalPosition::new(0, 0));
            }

            let _ = window.set_fullscreen(true);
            let _ = window.show();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenClaw activity screen");
}
