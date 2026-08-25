use std::sync::Mutex;
use tauri::{Emitter, State};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use redis::Client as RedisClient;
use chrono::Utc;
use dotenvy::dotenv;
use std::env;

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
    pub logged_in_user: Mutex<Option<User>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn register_passenger(
    display_name: String,
    email: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _pool = &state.db;
    Ok(format!("registered {} <{}>", display_name, email))
}

#[tauri::command]
async fn login_passenger(
    email: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<User, String> {
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

#[tauri::command]
async fn login_employee(
    employee_code: String,
    _password: String,
    state: State<'_, AppState>,
) -> Result<User, String> {
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

#[tauri::command]
async fn forgot_password(email: String, state: State<'_, AppState>) -> Result<String, String> {
    let new_password = "TmpP@ssw0rd123";
    let _ = &state.redis;
    Ok(format!(
        "A temporary password has been generated and (would be) emailed to {}: {}",
        email, new_password
    ))
}

#[tauri::command]
async fn send_notification(title: String, body: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<String, String> {
    let payload = serde_json::json!({ "title": title, "body": body, "ts": Utc::now() });
    if let Err(e) = app_handle.emit("saillintis:notification", payload.clone()) {
        eprintln!("emit error: {:?}", e);
    }

    let redis_client = &state.redis;
    let publish_result = tokio::spawn({
        let redis_client = redis_client.clone();
        let payload = payload.to_string();
        async move {
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                let _ : redis::RedisResult<i64> = redis::cmd("PUBLISH").arg("saillintis:notifications").arg(payload).query_async(&mut conn).await;
            }
        }
    });

    let _ = publish_result;
    Ok("sent".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect(">>> ERROR: DATABASE_URL is not set in your .env file! <<<");

    let db_pool = rt.block_on(async {
        println!(">>> Connecting to PostgreSQL 'SailLantis'...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect(">>> Failed to connect to PostgreSQL database! Check your .env file <<<");

        println!(">>> Executing SQL Migrations...");
        sqlx::migrate!("./sql/migrations")
            .run(&pool)
            .await
            .expect(">>> Failed to run SQL migrations! Check migration files <<<");

        println!(">>> SUCCESS: Database connected & migrations applied successfully! <<<");
        pool
    });

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