use crate::error::AppError;
use crate::models::account::{AccountType, MinecraftAccount};
use crate::services::account_service;
use crate::utils::paths::get_app_data_dir;
use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
use rsa::signature::{SignatureEncoding, Signer};
use rsa::RsaPrivateKey;
use serde::Deserialize;
use sha1::Sha1;
use std::path::PathBuf;
use std::sync::Arc;

/// Port the Yggdrasil server is running on (0 = not started)
pub static YGGDRASIL_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

/// Shared state for the Yggdrasil server
struct YggdrasilState {
    private_key: RsaPrivateKey,
    public_key_pem: String,
    skins_dir: PathBuf,
}

/// Get the path to the RSA keypair file
fn get_keypair_path() -> PathBuf {
    get_app_data_dir().join("yggdrasil_keypair.pem")
}

/// Get or create the RSA keypair for signing texture properties
fn get_or_create_keypair() -> Result<RsaPrivateKey, AppError> {
    let keypair_path = get_keypair_path();

    if keypair_path.exists() {
        let pem = std::fs::read_to_string(&keypair_path)?;
        RsaPrivateKey::from_pkcs8_pem(&pem)
            .map_err(|e| AppError::Internal(format!("Failed to load RSA keypair: {}", e)))
    } else {
        eprintln!("[yggdrasil] Generating new RSA keypair...");
        let mut rng = rsa::rand_core::OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 4096)
            .map_err(|e| AppError::Internal(format!("Failed to generate RSA key: {}", e)))?;

        // Save the private key
        if let Some(parent) = keypair_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AppError::Internal(format!("Failed to encode RSA key: {}", e)))?;
        std::fs::write(&keypair_path, pem.as_bytes())?;

        eprintln!("[yggdrasil] RSA keypair generated and saved");
        Ok(private_key)
    }
}

/// Get the skins storage directory
fn get_skins_dir() -> PathBuf {
    get_app_data_dir().join("skins")
}

/// Start the local Yggdrasil server
/// Returns the port it's listening on
pub async fn start_server() -> Result<u16, AppError> {
    let private_key = get_or_create_keypair()?;

    let public_key_pem = private_key
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| AppError::Internal(format!("Failed to encode public key: {}", e)))?;

    let skins_dir = get_skins_dir();
    std::fs::create_dir_all(&skins_dir)?;

    let state = Arc::new(YggdrasilState {
        private_key,
        public_key_pem,
        skins_dir,
    });

    let app = Router::new()
        .route(
            "/yggdrasil",
            get({
                let state = Arc::clone(&state);
                move || {
                    let state = Arc::clone(&state);
                    async move { server_metadata(state).await }
                }
            }),
        )
        .route(
            "/yggdrasil/",
            get({
                let state = Arc::clone(&state);
                move || {
                    let state = Arc::clone(&state);
                    async move { server_metadata(state).await }
                }
            }),
        )
        .route(
            "/yggdrasil/sessionserver/session/minecraft/join",
            post(join_server),
        )
        .route(
            "/yggdrasil/sessionserver/session/minecraft/hasJoined",
            get({
                let state = Arc::clone(&state);
                move |query| has_joined(query, state)
            }),
        )
        .route(
            "/yggdrasil/sessionserver/session/minecraft/profile/{uuid}",
            get({
                let state = Arc::clone(&state);
                move |path, query| get_profile(path, query, state)
            }),
        )
        .route(
            "/yggdrasil/textures/{hash}",
            get({
                let state = Arc::clone(&state);
                move |path| get_texture(path, state)
            }),
        );

    // Bind to a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::Internal(format!("Failed to bind Yggdrasil server: {}", e)))?;

    let port = listener
        .local_addr()
        .map_err(|e| AppError::Internal(format!("Failed to get local address: {}", e)))?
        .port();

    YGGDRASIL_PORT.store(port, std::sync::atomic::Ordering::Relaxed);

    eprintln!("[yggdrasil] Server starting on 127.0.0.1:{}", port);

    // Spawn the server in a background task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[yggdrasil] Server error: {}", e);
        }
    });

    Ok(port)
}

/// GET /yggdrasil/ - Server metadata
async fn server_metadata(state: Arc<YggdrasilState>) -> Json<serde_json::Value> {
    eprintln!("[yggdrasil] metadata request");
    Json(serde_json::json!({
        "meta": {
            "serverName": "ETLauncher Yggdrasil",
            "implementationName": "etlauncher",
            "implementationVersion": "1.0.0",
            "feature.non_email_login": true
        },
        "skinDomains": [
            "127.0.0.1",
            "localhost"
        ],
        "signaturePublickey": state.public_key_pem
    }))
}

/// POST /yggdrasil/sessionserver/session/minecraft/join - Always accept
async fn join_server() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct HasJoinedQuery {
    username: String,
    #[serde(rename = "serverId")]
    #[allow(dead_code)]
    server_id: Option<String>,
}

/// GET /yggdrasil/sessionserver/session/minecraft/hasJoined
async fn has_joined(Query(query): Query<HasJoinedQuery>, state: Arc<YggdrasilState>) -> Response {
    eprintln!(
        "[yggdrasil] hasJoined request for username: {}",
        query.username
    );
    match find_offline_account_by_username(&query.username) {
        Some(account) => {
            let port = YGGDRASIL_PORT.load(std::sync::atomic::Ordering::Relaxed);
            let profile = build_profile_response(&account, port, &state);
            Json(profile).into_response()
        }
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

#[derive(Deserialize)]
struct ProfileQuery {
    unsigned: Option<bool>,
}

/// GET /yggdrasil/sessionserver/session/minecraft/profile/:uuid
async fn get_profile(
    Path(uuid): Path<String>,
    Query(query): Query<ProfileQuery>,
    state: Arc<YggdrasilState>,
) -> Response {
    eprintln!("[yggdrasil] profile request for uuid: {}", uuid);
    // Normalize UUID (remove dashes)
    let uuid_normalized = uuid.replace('-', "");

    match find_offline_account_by_uuid(&uuid_normalized) {
        Some(account) => {
            let port = YGGDRASIL_PORT.load(std::sync::atomic::Ordering::Relaxed);
            let mut profile = build_profile_response(&account, port, &state);

            // If unsigned=true, remove signature
            if query.unsigned.unwrap_or(false) {
                if let Some(properties) = profile.get_mut("properties") {
                    if let Some(arr) = properties.as_array_mut() {
                        for prop in arr {
                            if let Some(obj) = prop.as_object_mut() {
                                obj.remove("signature");
                            }
                        }
                    }
                }
            }

            Json(profile).into_response()
        }
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

/// GET /yggdrasil/textures/:hash - Serve skin/cape PNG
async fn get_texture(Path(hash): Path<String>, state: Arc<YggdrasilState>) -> Response {
    eprintln!("[yggdrasil] texture request for hash: {}", hash);
    let texture_path = state.skins_dir.join(format!("{}.png", hash));

    match std::fs::read(&texture_path) {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "max-age=3600")
            .body(Body::from(data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Find an offline account by username
fn find_offline_account_by_username(username: &str) -> Option<MinecraftAccount> {
    let accounts = account_service::load_accounts().ok()?;
    accounts.into_iter().find(|a| {
        a.account_type == AccountType::Offline && a.username.eq_ignore_ascii_case(username)
    })
}

/// Find an offline account by UUID
fn find_offline_account_by_uuid(uuid: &str) -> Option<MinecraftAccount> {
    let accounts = account_service::load_accounts().ok()?;
    let uuid_normalized = uuid.replace('-', "");
    accounts.into_iter().find(|a| {
        a.account_type == AccountType::Offline && a.uuid.replace('-', "") == uuid_normalized
    })
}

/// Build a profile response JSON with signed texture properties
fn build_profile_response(
    account: &MinecraftAccount,
    port: u16,
    state: &YggdrasilState,
) -> serde_json::Value {
    let uuid_normalized = account.uuid.replace('-', "");

    // Build textures payload
    let mut textures: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    if let Some(ref skin_hash) = account.offline_skin_hash {
        let variant = account.offline_skin_variant.as_deref().unwrap_or("classic");
        let mut skin_obj = serde_json::json!({
            "url": format!("http://127.0.0.1:{}/yggdrasil/textures/{}", port, skin_hash)
        });
        if variant == "slim" {
            skin_obj["metadata"] = serde_json::json!({
                "model": "slim"
            });
        }
        textures.insert("SKIN".to_string(), skin_obj);
    }

    if let Some(ref cape_hash) = account.offline_cape_hash {
        textures.insert(
            "CAPE".to_string(),
            serde_json::json!({
                "url": format!("http://127.0.0.1:{}/yggdrasil/textures/{}", port, cape_hash)
            }),
        );
    }

    let texture_property_value = serde_json::json!({
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "profileId": uuid_normalized,
        "profileName": account.username,
        "textures": textures
    });

    let texture_value_str = serde_json::to_string(&texture_property_value).unwrap_or_default();
    let texture_value_b64 = BASE64.encode(texture_value_str.as_bytes());

    // Sign the texture value
    let signing_key = SigningKey::<Sha1>::new(state.private_key.clone());
    let signature = signing_key.sign(texture_value_b64.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());

    serde_json::json!({
        "id": uuid_normalized,
        "name": account.username,
        "properties": [
            {
                "name": "textures",
                "value": texture_value_b64,
                "signature": signature_b64
            }
        ]
    })
}

/// Get the current Yggdrasil server port (0 if not running)
pub fn get_port() -> u16 {
    YGGDRASIL_PORT.load(std::sync::atomic::Ordering::Relaxed)
}
