// Core Tauri application scaffold for saILintis project.
// This file registers shared app state (Postgres pool, Redis client, logged-in user) and exposes command stubs.

use std::sync::Mutex;

use tauri::State;

use tauri::Emitter;

use serde::{Deserialize, Serialize};

use sqlx::PgPool;
use redis::Client as RedisClient;

use chrono::Utc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub display_name: String,
    pub email: Option<String>,
    pub role: String,
}

pub struct AppState {
    pub db: PgPool,
    pub redis: RedisClient,
    // Simple in-memory logged-in user holder for the active window instance; real apps should use session tokens
    pub logged_in_user: Mutex<Option<User>>,
}

// Example command: simple greet
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Passenger registration stub
#[tauri::command]
async fn register_passenger(
    display_name: String,
    email: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // TODO: validate inputs, check for duplicate email, hash password with Argon2 + salt, insert into Postgres.
    // This is a scaffold showing where DB/Redis usage would occur.
    let _pool = &state.db;
    // Example (unexecuted) query placeholder:
    // sqlx::query!("INSERT INTO passengers (display_name, email, password_hash) VALUES ($1,$2,$3)", display_name, email, hash)
    //     .execute(_pool).await.map_err(|e| e.to_string())?;
    Ok(format!("registered {} <{}>", display_name, email))
}

// Passenger login stub (email + password)
#[tauri::command]
async fn login_passenger(
    email: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<User, String> {
    // TODO: lookup user by email, verify Argon2 hash, then set logged_in_user state
    // For now return a fake user for scaffold
    let user = User {
        id: 1,
        display_name: "Demo Passenger".into(),
        email: Some(email.clone()),
        role: "passenger".into(),
    };
    let mut guard = state.logged_in_user.lock().map_err(|e| e.to_string())?;
    *guard = Some(user.clone());
    Ok(user)
}

// Employee login stub (employee code + password)
#[tauri::command]
async fn login_employee(
    employee_code: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<User, String> {
    // TODO: verify employee credentials from DB, enforce RBAC mapping
    let user = User {
        id: 100,
        display_name: format!("Employee {}", employee_code),
        email: None,
        role: "staff".into(),
    };
    let mut guard = state.logged_in_user.lock().map_err(|e| e.to_string())?;
    *guard = Some(user.clone());
    Ok(user)
}

// Forgot password stub: generate secure random password and (in the real app) email it
#[tauri::command]
async fn forgot_password(email: String, state: State<'_, AppState>) -> Result<String, String> {
    // TODO: generate strong password, update DB with hashed password, send email via SMTP microservice or external API
    let new_password = "TmpP@ssw0rd123"; // placeholder; generate securely in real app
    // Example: use Redis to store short-lived reset token if desired
    let _ = &state.redis;
    Ok(format!(
        "A temporary password has been generated and (would be) emailed to {}: {}",
        email, new_password
    ))
}

// Notifications: emit an event to all windows and optionally publish to Redis pub/sub
#[tauri::command]
async fn send_notification(title: String, body: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<String, String> {
    // Emit a Tauri event to all windows so frontends can react in real-time
    let payload = serde_json::json!({ "title": title, "body": body, "ts": Utc::now() });
    if let Err(e) = app_handle.emit("saillintis:notification", payload.clone()) {
        eprintln!("emit error: {:?}", e);
    }

    // Publish to Redis channel for multi-instance sync (if configured)
    // This is a non-blocking best-effort publish; real app should await and handle errors
    let redis_client = &state.redis;
    let publish_result = tokio::spawn({
        let redis_client = redis_client.clone();
        let payload = payload.to_string();
        async move {
            // Attempt to get an async connection and publish
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                let _ : redis::RedisResult<i64> = redis::cmd("PUBLISH").arg("saillintis:notifications").arg(payload).query_async(&mut conn).await;
            }
        }
    });

    // Detach the publish task (best-effort)
    let _ = publish_result;

    Ok("sent".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize async runtime for initializers
    // Create DB pool and Redis client here — in real use read DSNs from environment/config
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    // Basic placeholders — replace DSNs with real values or environment variables
    let db_pool = rt
        .block_on(async {
            // Example: PgPool::connect("******localhost/saILintis").await
            PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/saillintis")
        })
        .expect("Failed to create connection pool with valid URL");

    let redis_client = rt
        .block_on(async { RedisClient::open("redis://127.0.0.1/") })
        .expect("failed to create redis client");

    let app_state = AppState {
        db: db_pool,
        redis: redis_client,
        logged_in_user: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            register_passenger,
            login_passenger,
            login_employee,
            forgot_password,
            send_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
