use axum::{
    routing::{get, post},
    Router, Json, extract::State, extract::Query, response::Redirect,
    http::{StatusCode},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use oauth2::{
    basic::BasicClient, AuthUrl, TokenUrl, RedirectUrl, AuthorizationCode,
    ClientId, ClientSecret, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::ServeDir;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::process::Command;
use std::{fs, env};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::RngCore;

const SESSION_COOKIE_NAME: &str = "session_id";
const SESSION_DURATION_DAYS: i64 = 7;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    oauth_client: BasicClient,
    http_client: reqwest::Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskRow {
    customer: String,
    loading_point: String,
    unloading_point: String,
    cargo: String,
    trips: String,
    distance: String,
    tons: String,
    arrival_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchItem {
    driver_id: i32,
    vehicle_id: i32,
    date: Option<String>,
    tasks: Vec<TaskRow>,
    tractor_mode: Option<String>,
    work_name: Option<String>,
    trailer: Option<String>,
    no_date: Option<bool>,
    no_time: Option<bool>,
    medical_exam_time: Option<String>,
    departure_time: Option<String>,
    return_time: Option<String>,
    fuel_brand: Option<String>,
    fuel_code: Option<String>,
    fuel_issued: Option<String>,
    fuel_remain_depart: Option<String>,
    fuel_remain_return: Option<String>,
    fuel_submitted: Option<String>,
    fuel_coeff: Option<String>,
    fuel_special: Option<String>,
    fuel_engine: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchRequest {
    items: Vec<PrintBatchItem>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct DefaultValues {
    id: i32,
    user_id: i64,
    customer: Option<String>,
    loading_point: Option<String>,
    unloading_point: Option<String>,
    cargo: Option<String>,
    trips: Option<String>,
    distance: Option<String>,
    tons: Option<String>,
    arrival_time: Option<String>,
    field_object: Option<String>,
    field_area: Option<String>,
    field_norm: Option<String>,
    field_fact: Option<String>,
    field_motohours: Option<String>,
    medical_exam_time: Option<String>,
    departure_time: Option<String>,
    return_time: Option<String>,
    fuel_brand: Option<String>,
    fuel_code: Option<String>,
    fuel_issued: Option<String>,
    fuel_remain_depart: Option<String>,
    fuel_remain_return: Option<String>,
    fuel_submitted: Option<String>,
    fuel_coeff: Option<String>,
    fuel_special: Option<String>,
    fuel_engine: Option<String>,
    work_name: Option<String>,
    trailer: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct CompanySettings {
    id: i32,
    user_id: i64,
    company_name: Option<String>,
    company_address: Option<String>,
    company_inn: Option<String>,
    dispatcher_name: Option<String>,
    mechanic_name: Option<String>,
    medic_name: Option<String>,
}

#[derive(Serialize)]
struct PrintResponse {
    success: bool,
    message: String,
    pdf_url: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct Driver { 
    id: i32, 
    user_id: i64,
    name: String, 
    driving_license: Option<String>,
    driving_license_date: Option<String>,
    tractor_license: Option<String>,
    tractor_license_date: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct Vehicle { 
    id: i32, 
    user_id: i64,
    name: String, 
    license_plate: Option<String>,
    sts: Option<String>,
    vehicle_type: Option<String>,
    category: Option<String>
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct User {
    id: i64,
    yandex_id: String,
    email: Option<String>,
    name: Option<String>,
    avatar: Option<String>,
    created_at: i64,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct Session {
    id: String,
    user_id: i64,
    created_at: i64,
    expires_at: i64,
}

fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .expect("Не удалось получить путь к исполняемому файлу")
        .parent()
        .expect("Не удалось получить родительскую директорию")
        .to_path_buf()
}

fn setup_typst_binary() {
    let exe_dir = get_exe_dir();
    let (src_name, target_name) = if cfg!(windows) {
        ("bin/typst-windows.exe", "typst.exe")
    } else {
        ("bin/typst-linux", "typst")
    };

    let target_path = exe_dir.join(target_name);
    
    if !target_path.exists() {
        println!("Копирование бинарника Typst: {:?}", target_path);
        if let Err(e) = fs::copy(src_name, &target_path) {
            eprintln!("Ошибка при копировании Typst: {}", e);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&target_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&target_path, perms).unwrap();
            }
        }
    }
}

fn generate_session_id() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: Option<String>,
}

#[derive(Deserialize, Debug)]
struct YandexUserInfo {
    id: String,
    #[serde(default)]
    default_email: Option<String>,
    #[serde(default)]
    real_name: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    default_avatar_id: Option<String>,
}

async fn yandex_login(
    State(state): State<AppState>,
) -> Redirect {
    let (auth_url, _csrf_token) = state.oauth_client
        .authorize_url(oauth2::CsrfToken::new_random)
        .url();
    
    Redirect::to(auth_url.as_str())
}

async fn yandex_callback(
    State(state): State<AppState>,
    Query(params): Query<AuthRequest>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), (StatusCode, String)> {
    let token = match state.oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await {
        Ok(token) => token,
        Err(e) => {
            eprintln!("OAuth error: {}", e);
            return Err((StatusCode::BAD_REQUEST, "OAuth error".to_string()));
        }
    };

    let access_token = token.access_token().secret();
    
    let user_info: YandexUserInfo = match state.http_client
        .get("https://login.yandex.ru/info?format=json")
        .header("Authorization", format!("OAuth {}", access_token))
        .send()
        .await {
        Ok(resp) => match resp.json().await {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Failed to parse user info: {}", e);
                return Err((StatusCode::BAD_REQUEST, "User info error".to_string()));
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch user info: {}", e);
            return Err((StatusCode::BAD_REQUEST, "User info error".to_string()));
        }
    };

    let user: User = match sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE yandex_id = ?"
    )
    .bind(&user_info.id)
    .fetch_optional(&state.db)
    .await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let name = user_info.real_name
                .or(user_info.display_name)
                .unwrap_or_else(|| "User".to_string());
            
            let avatar = user_info.default_avatar_id
                .map(|id| format!("https://avatars.yandex.net/get-yapic/{}/islands-200", id));

            let id = sqlx::query(
                "INSERT INTO users (yandex_id, email, name, avatar, created_at) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&user_info.id)
            .bind(&user_info.default_email)
            .bind(&name)
            .bind(&avatar)
            .bind(current_timestamp())
            .execute(&state.db)
            .await
            .map_err(|e| {
                eprintln!("DB error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            })?
            .last_insert_rowid();

            User {
                id,
                yandex_id: user_info.id,
                email: user_info.default_email,
                name: Some(name),
                avatar,
                created_at: current_timestamp(),
            }
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()));
        }
    };

    let session_id = generate_session_id();
    let expires_at = current_timestamp() + (SESSION_DURATION_DAYS * 24 * 60 * 60);

    sqlx::query(
        "INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&session_id)
    .bind(user.id)
    .bind(current_timestamp())
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Session error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Session error".to_string())
    })?;

    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_id))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::days(SESSION_DURATION_DAYS));

    let jar = jar.add(cookie);
    
    Ok((jar, Redirect::to("/")))
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), (StatusCode, String)> {
    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_cookie.value())
            .execute(&state.db)
            .await
            .ok();
    }

    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::seconds(0));

    let jar = jar.add(cookie);
    Ok((jar, Redirect::to("/")))
}

async fn get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let session_id = match jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "Not authenticated"}))));
        }
    };

    let session: Session = match sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = ? AND expires_at > ?"
    )
    .bind(&session_id)
    .bind(current_timestamp())
    .fetch_optional(&state.db)
    .await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "Session expired"}))));
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"}))));
        }
    };

    let user: User = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(session.user_id)
        .fetch_optional(&state.db)
        .await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "User not found"}))));
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"}))));
        }
    };

    Ok(Json(json!({
        "id": user.id,
        "name": user.name,
        "email": user.email,
        "avatar": user.avatar,
    })))
}

async fn get_user_id_from_session(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<i64, (StatusCode, String)> {
    let session_id = match jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()));
        }
    };

    let session: Session = match sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = ? AND expires_at > ?"
    )
    .bind(&session_id)
    .bind(current_timestamp())
    .fetch_optional(&state.db)
    .await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()));
        }
    };

    Ok(session.user_id)
}

async fn get_drivers(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<Driver>>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let drivers = sqlx::query_as::<_, Driver>(
        "SELECT * FROM drivers WHERE user_id = ? ORDER BY name"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(drivers))
}

#[derive(Deserialize)]
struct CreateDriverReq { 
    name: String, 
    driving_license: Option<String>,
    driving_license_date: Option<String>,
    tractor_license: Option<String>,
    tractor_license_date: Option<String>,
}

#[derive(Deserialize)]
struct UpdateDriverReq { 
    id: i32, 
    name: String, 
    driving_license: Option<String>,
    driving_license_date: Option<String>,
    tractor_license: Option<String>,
    tractor_license_date: Option<String>,
}

#[derive(Deserialize)]
struct DeleteReq { id: i32 }

async fn create_driver(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateDriverReq>,
) -> Result<Json<Driver>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let id = sqlx::query(
        "INSERT INTO drivers (user_id, name, driving_license, driving_license_date, tractor_license, tractor_license_date) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.driving_license)
    .bind(&payload.driving_license_date)
    .bind(&payload.tractor_license)
    .bind(&payload.tractor_license_date)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .last_insert_rowid() as i32;
    
    Ok(Json(Driver { 
        id, 
        user_id,
        name: payload.name, 
        driving_license: payload.driving_license,
        driving_license_date: payload.driving_license_date,
        tractor_license: payload.tractor_license,
        tractor_license_date: payload.tractor_license_date,
    }))
}

async fn update_driver(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<UpdateDriverReq>,
) -> Result<Json<Driver>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query(
        "UPDATE drivers SET name = ?, driving_license = ?, driving_license_date = ?, tractor_license = ?, tractor_license_date = ? WHERE id = ? AND user_id = ?"
    )
    .bind(&payload.name)
    .bind(&payload.driving_license)
    .bind(&payload.driving_license_date)
    .bind(&payload.tractor_license)
    .bind(&payload.tractor_license_date)
    .bind(payload.id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(Driver { 
        id: payload.id, 
        user_id,
        name: payload.name, 
        driving_license: payload.driving_license,
        driving_license_date: payload.driving_license_date,
        tractor_license: payload.tractor_license,
        tractor_license_date: payload.tractor_license_date,
    }))
}

async fn delete_driver(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<DeleteReq>,
) -> Result<Json<()>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query("DELETE FROM drivers WHERE id = ? AND user_id = ?")
        .bind(payload.id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;
    
    Ok(Json(()))
}

async fn get_vehicles(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<Vehicle>>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let vehicles = sqlx::query_as::<_, Vehicle>(
        "SELECT * FROM vehicles WHERE user_id = ? ORDER BY name"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(vehicles))
}

#[derive(Deserialize)]
struct CreateVehicleReq { 
    name: String, 
    license_plate: Option<String>, 
    sts: Option<String>, 
    vehicle_type: Option<String>, 
    category: Option<String> 
}

#[derive(Deserialize)]
struct UpdateVehicleReq { 
    id: i32, 
    name: String, 
    license_plate: Option<String>, 
    sts: Option<String>, 
    vehicle_type: Option<String>, 
    category: Option<String> 
}

async fn create_vehicle(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateVehicleReq>,
) -> Result<Json<Vehicle>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let id = sqlx::query(
        "INSERT INTO vehicles (user_id, name, license_plate, sts, vehicle_type, category) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.license_plate)
    .bind(&payload.sts)
    .bind(&payload.vehicle_type)
    .bind(&payload.category)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?
    .last_insert_rowid() as i32;
    
    Ok(Json(Vehicle { 
        id, 
        user_id,
        name: payload.name, 
        license_plate: payload.license_plate, 
        sts: payload.sts, 
        vehicle_type: payload.vehicle_type, 
        category: payload.category 
    }))
}

async fn update_vehicle(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<UpdateVehicleReq>,
) -> Result<Json<Vehicle>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query(
        "UPDATE vehicles SET name = ?, license_plate = ?, sts = ?, vehicle_type = ?, category = ? WHERE id = ? AND user_id = ?"
    )
    .bind(&payload.name)
    .bind(&payload.license_plate)
    .bind(&payload.sts)
    .bind(&payload.vehicle_type)
    .bind(&payload.category)
    .bind(payload.id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(Vehicle { 
        id: payload.id, 
        user_id,
        name: payload.name, 
        license_plate: payload.license_plate, 
        sts: payload.sts, 
        vehicle_type: payload.vehicle_type, 
        category: payload.category 
    }))
}

async fn delete_vehicle(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<DeleteReq>,
) -> Result<Json<()>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query("DELETE FROM vehicles WHERE id = ? AND user_id = ?")
        .bind(payload.id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;
    
    Ok(Json(()))
}

#[derive(Deserialize)]
struct SaveSettingsReq {
    company_name: Option<String>,
    company_address: Option<String>,
    company_inn: Option<String>,
    dispatcher_name: Option<String>,
    mechanic_name: Option<String>,
    medic_name: Option<String>,
}

async fn get_settings(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<CompanySettings>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let settings: CompanySettings = sqlx::query_as(
        "SELECT * FROM company_settings WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(settings))
}

async fn save_settings(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<SaveSettingsReq>,
) -> Result<Json<CompanySettings>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query(
        "INSERT INTO company_settings (user_id, company_name, company_address, company_inn, dispatcher_name, mechanic_name, medic_name) 
         VALUES (?, ?, ?, ?, ?, ?, ?) 
         ON CONFLICT(user_id) DO UPDATE SET 
         company_name=excluded.company_name, company_address=excluded.company_address, company_inn=excluded.company_inn, 
         dispatcher_name=excluded.dispatcher_name, mechanic_name=excluded.mechanic_name, medic_name=excluded.medic_name"
    )
    .bind(user_id)
    .bind(&payload.company_name)
    .bind(&payload.company_address)
    .bind(&payload.company_inn)
    .bind(&payload.dispatcher_name)
    .bind(&payload.mechanic_name)
    .bind(&payload.medic_name)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(CompanySettings {
        id: 1,
        user_id,
        company_name: payload.company_name,
        company_address: payload.company_address,
        company_inn: payload.company_inn,
        dispatcher_name: payload.dispatcher_name,
        mechanic_name: payload.mechanic_name,
        medic_name: payload.medic_name,
    }))
}

#[derive(Deserialize)]
struct SaveDefaultsReq {
    customer: Option<String>,
    loading_point: Option<String>,
    unloading_point: Option<String>,
    cargo: Option<String>,
    trips: Option<String>,
    distance: Option<String>,
    tons: Option<String>,
    arrival_time: Option<String>,
    field_object: Option<String>,
    field_area: Option<String>,
    field_norm: Option<String>,
    field_fact: Option<String>,
    field_motohours: Option<String>,
    medical_exam_time: Option<String>,
    departure_time: Option<String>,
    return_time: Option<String>,
    fuel_brand: Option<String>,
    fuel_code: Option<String>,
    fuel_issued: Option<String>,
    fuel_remain_depart: Option<String>,
    fuel_remain_return: Option<String>,
    fuel_submitted: Option<String>,
    fuel_coeff: Option<String>,
    fuel_special: Option<String>,
    fuel_engine: Option<String>,
    work_name: Option<String>,
    trailer: Option<String>,
}

async fn get_defaults(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<DefaultValues>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let defaults: DefaultValues = sqlx::query_as(
        "SELECT * FROM default_values WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(defaults))
}

async fn save_defaults(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<SaveDefaultsReq>,
) -> Result<Json<DefaultValues>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    sqlx::query(
        "INSERT INTO default_values (user_id, customer, loading_point, unloading_point, cargo, trips, distance, tons, arrival_time, field_object, field_area, field_norm, field_fact, field_motohours, medical_exam_time, departure_time, return_time, fuel_brand, fuel_code, fuel_issued, fuel_remain_depart, fuel_remain_return, fuel_submitted, fuel_coeff, fuel_special, fuel_engine, work_name, trailer) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
         ON CONFLICT(user_id) DO UPDATE SET 
         customer=excluded.customer, loading_point=excluded.loading_point, unloading_point=excluded.unloading_point, 
         cargo=excluded.cargo, trips=excluded.trips, distance=excluded.distance, tons=excluded.tons, arrival_time=excluded.arrival_time,
         field_object=excluded.field_object, field_area=excluded.field_area, field_norm=excluded.field_norm, 
         field_fact=excluded.field_fact, field_motohours=excluded.field_motohours,
         medical_exam_time=excluded.medical_exam_time, departure_time=excluded.departure_time, return_time=excluded.return_time,
         fuel_brand=excluded.fuel_brand, fuel_code=excluded.fuel_code, fuel_issued=excluded.fuel_issued,
         fuel_remain_depart=excluded.fuel_remain_depart, fuel_remain_return=excluded.fuel_remain_return,
         fuel_submitted=excluded.fuel_submitted, fuel_coeff=excluded.fuel_coeff,
         fuel_special=excluded.fuel_special, fuel_engine=excluded.fuel_engine,
         work_name=excluded.work_name, trailer=excluded.trailer"
    )
    .bind(user_id)
    .bind(&payload.customer)
    .bind(&payload.loading_point)
    .bind(&payload.unloading_point)
    .bind(&payload.cargo)
    .bind(&payload.trips)
    .bind(&payload.distance)
    .bind(&payload.tons)
    .bind(&payload.arrival_time)
    .bind(&payload.field_object)
    .bind(&payload.field_area)
    .bind(&payload.field_norm)
    .bind(&payload.field_fact)
    .bind(&payload.field_motohours)
    .bind(&payload.medical_exam_time)
    .bind(&payload.departure_time)
    .bind(&payload.return_time)
    .bind(&payload.fuel_brand)
    .bind(&payload.fuel_code)
    .bind(&payload.fuel_issued)
    .bind(&payload.fuel_remain_depart)
    .bind(&payload.fuel_remain_return)
    .bind(&payload.fuel_submitted)
    .bind(&payload.fuel_coeff)
    .bind(&payload.fuel_special)
    .bind(&payload.fuel_engine)
    .bind(&payload.work_name)
    .bind(&payload.trailer)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    Ok(Json(DefaultValues {
        id: 1,
        user_id,
        customer: payload.customer,
        loading_point: payload.loading_point,
        unloading_point: payload.unloading_point,
        cargo: payload.cargo,
        trips: payload.trips,
        distance: payload.distance,
        tons: payload.tons,
        arrival_time: payload.arrival_time,
        field_object: payload.field_object,
        field_area: payload.field_area,
        field_norm: payload.field_norm,
        field_fact: payload.field_fact,
        field_motohours: payload.field_motohours,
        medical_exam_time: payload.medical_exam_time,
        departure_time: payload.departure_time,
        return_time: payload.return_time,
        fuel_brand: payload.fuel_brand,
        fuel_code: payload.fuel_code,
        fuel_issued: payload.fuel_issued,
        fuel_remain_depart: payload.fuel_remain_depart,
        fuel_remain_return: payload.fuel_remain_return,
        fuel_submitted: payload.fuel_submitted,
        fuel_coeff: payload.fuel_coeff,
        fuel_special: payload.fuel_special,
        fuel_engine: payload.fuel_engine,
        work_name: payload.work_name,
        trailer: payload.trailer,
    }))
}

fn escape_typst(s: &str) -> String {
    s.replace("\\", "\\\\").replace("\"", "\\\"")
}

fn format_tasks_typst(tasks: &[TaskRow]) -> String {
    let mut parts = Vec::new();
    for task in tasks.iter().take(3) {
        parts.push(format!(
            r#"(customer: "{}", loading_point: "{}", unloading_point: "{}", cargo: "{}", trips: "{}", distance: "{}", tons: "{}", arrival_time: "{}")"#,
            escape_typst(&task.customer),
            escape_typst(&task.loading_point),
            escape_typst(&task.unloading_point),
            escape_typst(&task.cargo),
            escape_typst(&task.trips),
            escape_typst(&task.distance),
            escape_typst(&task.tons),
            escape_typst(&task.arrival_time)
        ));
    }
    for _ in parts.len()..3 {
        parts.push(r#"(customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: "")"#.to_string());
    }
    format!("({})", parts.join(", "))
}

async fn print_batch(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<PrintBatchRequest>,
) -> Result<Json<PrintResponse>, (StatusCode, String)> {
    let user_id = get_user_id_from_session(State(state.clone()), jar).await?;
    
    let template_base = fs::read_to_string("templates/template.typ").map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Template not found".to_string())
    })?;
    
    let settings: CompanySettings = sqlx::query_as(
        "SELECT * FROM company_settings WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("DB error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
    
    let mut compiled_typst = String::new();
    
    for (i, item) in payload.items.iter().enumerate() {
        let driver: Driver = sqlx::query_as(
            "SELECT * FROM drivers WHERE id = ? AND user_id = ?"
        )
        .bind(item.driver_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Driver not found".to_string()))?;
        
        let vehicle: Vehicle = sqlx::query_as(
            "SELECT * FROM vehicles WHERE id = ? AND user_id = ?"
        )
        .bind(item.vehicle_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Vehicle not found".to_string()))?;
        
        let drv_license = driver.driving_license.unwrap_or_default();
        let drv_license_date = driver.driving_license_date.unwrap_or_default();
        let tr_license = driver.tractor_license.unwrap_or_default();
        let tr_license_date = driver.tractor_license_date.unwrap_or_default();
        
        let license_plate = vehicle.license_plate.unwrap_or_default();
        let sts = vehicle.sts.unwrap_or_default();
        let vehicle_type = vehicle.vehicle_type.unwrap_or_else(|| "Грузовой".to_string());
        
        let date = item.date.clone().unwrap_or_else(|| {
            let now = chrono::Local::now();
            now.format("%d.%m.%Y").to_string()
        });
        
        let tractor_mode_val = item.tractor_mode.clone().unwrap_or_else(|| "cargo".to_string());
        let category = vehicle.category.unwrap_or_default();
        let work_name = item.work_name.clone().unwrap_or_default();
        let no_date = item.no_date.unwrap_or(false);
        let no_time = item.no_time.unwrap_or(false);
        
        let is_tractor = vehicle_type == "Трактор";
        
        // Получаем значения по умолчанию для времени и горючего
        let defaults: DefaultValues = sqlx::query_as(
            "SELECT * FROM default_values WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?
        .unwrap_or(DefaultValues {
            id: 1,
            user_id,
            customer: None,
            loading_point: None,
            unloading_point: None,
            cargo: None,
            trips: None,
            distance: None,
            tons: None,
            arrival_time: None,
            field_object: None,
            field_area: None,
            field_norm: None,
            field_fact: None,
            field_motohours: None,
            medical_exam_time: Some("07:30".to_string()),
            departure_time: None,
            return_time: None,
            fuel_brand: Some("дизель".to_string()),
            fuel_code: None,
            fuel_issued: None,
            fuel_remain_depart: None,
            fuel_remain_return: None,
            fuel_submitted: None,
            fuel_coeff: None,
            fuel_special: None,
            fuel_engine: None,
            work_name: None,
            trailer: None,
        });
        
        // Используем данные из формы, если есть, иначе из дефолтов
        let medical_exam_time_val = item.medical_exam_time.clone().or(defaults.medical_exam_time.clone()).unwrap_or_else(|| "07:30".to_string());
        let departure_time_val = item.departure_time.clone().or(defaults.departure_time.clone()).unwrap_or_default();
        let return_time_val = item.return_time.clone().or(defaults.return_time.clone()).unwrap_or_default();
        let fuel_brand_val = item.fuel_brand.clone().or(defaults.fuel_brand.clone()).unwrap_or_else(|| "дизель".to_string());
        let fuel_code_val = item.fuel_code.clone().or(defaults.fuel_code.clone()).unwrap_or_default();
        let fuel_issued_val = item.fuel_issued.clone().or(defaults.fuel_issued.clone()).unwrap_or_default();
        let fuel_remain_depart_val = item.fuel_remain_depart.clone().or(defaults.fuel_remain_depart.clone()).unwrap_or_default();
        let fuel_remain_return_val = item.fuel_remain_return.clone().or(defaults.fuel_remain_return.clone()).unwrap_or_default();
        let fuel_submitted_val = item.fuel_submitted.clone().or(defaults.fuel_submitted.clone()).unwrap_or_default();
        let fuel_coeff_val = item.fuel_coeff.clone().or(defaults.fuel_coeff.clone()).unwrap_or_default();
        let fuel_special_val = item.fuel_special.clone().or(defaults.fuel_special.clone()).unwrap_or_default();
        let fuel_engine_val = item.fuel_engine.clone().or(defaults.fuel_engine.clone()).unwrap_or_default();
        
        // Формируем вызов в зависимости от типа ТС
        let call = if is_tractor {
            format!(r#"
#waybill_tractor(
  driver: "{}",
  driving_license: "{}",
  driving_license_date: "{}",
  tractor_license: "{}",
  tractor_license_date: "{}",
  vehicle: "{}",
  license_plate: "{}",
  sts: "{}",
  category: "{}",
  date: "{}",
  company_name: "{}",
  company_address: "{}",
  company_inn: "{}",
  dispatcher_name: "{}",
  mechanic_name: "{}",
  medic_name: "{}",
  medical_exam_time: "{}",
  departure_time: "{}",
  return_time: "{}",
  fuel_brand: "{}",
  fuel_code: "{}",
  fuel_issued: "{}",
  fuel_remain_depart: "{}",
  fuel_remain_return: "{}",
  fuel_submitted: "{}",
  fuel_coeff: "{}",
  fuel_special: "{}",
  fuel_engine: "{}",
  work_name: "{}",
  trailer: "{}",
  tractor_mode: "{}",
  no_date: {},
  no_time: {},
  tasks: {}
)
"#,
            escape_typst(&driver.name),
            escape_typst(&drv_license),
            escape_typst(&drv_license_date),
            escape_typst(&tr_license),
            escape_typst(&tr_license_date),
            escape_typst(&vehicle.name),
            escape_typst(&license_plate),
            escape_typst(&sts),
            escape_typst(&category),
            escape_typst(&date),
            escape_typst(&settings.company_name.clone().unwrap_or_default()),
            escape_typst(&settings.company_address.clone().unwrap_or_default()),
            escape_typst(&settings.company_inn.clone().unwrap_or_default()),
            escape_typst(&settings.dispatcher_name.clone().unwrap_or_default()),
            escape_typst(&settings.mechanic_name.clone().unwrap_or_default()),
            escape_typst(&settings.medic_name.clone().unwrap_or_default()),
            escape_typst(&medical_exam_time_val),
            escape_typst(&departure_time_val),
            escape_typst(&return_time_val),
            escape_typst(&fuel_brand_val),
            escape_typst(&fuel_code_val),
            escape_typst(&fuel_issued_val),
            escape_typst(&fuel_remain_depart_val),
            escape_typst(&fuel_remain_return_val),
            escape_typst(&fuel_submitted_val),
            escape_typst(&fuel_coeff_val),
            escape_typst(&fuel_special_val),
            escape_typst(&fuel_engine_val),
            escape_typst(&work_name),
            escape_typst(&item.trailer.clone().unwrap_or_default()),
            escape_typst(&tractor_mode_val),
            no_date,
            no_time,
            format_tasks_typst(&item.tasks)
        )
        } else {
            format!(r#"
#waybill_truck(
  driver: "{}",
  driving_license: "{}",
  driving_license_date: "{}",
  tractor_license: "{}",
  tractor_license_date: "{}",
  vehicle: "{}",
  license_plate: "{}",
  sts: "{}",
  category: "{}",
  date: "{}",
  company_name: "{}",
  company_address: "{}",
  company_inn: "{}",
  dispatcher_name: "{}",
  mechanic_name: "{}",
  medic_name: "{}",
  medical_exam_time: "{}",
  departure_time: "{}",
  return_time: "{}",
  fuel_brand: "{}",
  fuel_code: "{}",
  fuel_issued: "{}",
  fuel_remain_depart: "{}",
  fuel_remain_return: "{}",
  fuel_submitted: "{}",
  fuel_coeff: "{}",
  fuel_special: "{}",
  fuel_engine: "{}",
  no_date: {},
  no_time: {},
  tasks: {}
)
"#,
            escape_typst(&driver.name),
            escape_typst(&drv_license),
            escape_typst(&drv_license_date),
            escape_typst(&tr_license),
            escape_typst(&tr_license_date),
            escape_typst(&vehicle.name),
            escape_typst(&license_plate),
            escape_typst(&sts),
            escape_typst(&category),
            escape_typst(&date),
            escape_typst(&settings.company_name.clone().unwrap_or_default()),
            escape_typst(&settings.company_address.clone().unwrap_or_default()),
            escape_typst(&settings.company_inn.clone().unwrap_or_default()),
            escape_typst(&settings.dispatcher_name.clone().unwrap_or_default()),
            escape_typst(&settings.mechanic_name.clone().unwrap_or_default()),
            escape_typst(&settings.medic_name.clone().unwrap_or_default()),
            escape_typst(&medical_exam_time_val),
            escape_typst(&departure_time_val),
            escape_typst(&return_time_val),
            escape_typst(&fuel_brand_val),
            escape_typst(&fuel_code_val),
            escape_typst(&fuel_issued_val),
            escape_typst(&fuel_remain_depart_val),
            escape_typst(&fuel_remain_return_val),
            escape_typst(&fuel_submitted_val),
            escape_typst(&fuel_coeff_val),
            escape_typst(&fuel_special_val),
            escape_typst(&fuel_engine_val),
            no_date,
            no_time,
            format_tasks_typst(&item.tasks)
        )
        };
        
        if i > 0 {
            compiled_typst.push_str("\n#pagebreak()\n");
        }
        compiled_typst.push_str(&call);
    }
    
    let temp_typ = "data/temp_batch.typ";
    fs::write(temp_typ, format!("{}\n{}", template_base, compiled_typst)).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Write error: {}", e))
    })?;

    let typst_exe = get_exe_dir().join(if cfg!(windows) { "typst.exe" } else { "typst" });
    let output = Command::new(typst_exe)
        .arg("compile")
        .arg(temp_typ)
        .arg("static/out_batch.pdf")
        .output();
        
    let _ = fs::remove_file(temp_typ);

    match output {
        Ok(out) if out.status.success() => Ok(Json(PrintResponse { 
            success: true, 
            message: "OK".to_string(), 
            pdf_url: Some("/out_batch.pdf".to_string()) 
        })),
        Ok(out) => Ok(Json(PrintResponse { 
            success: false, 
            message: format!("Ошибка: {}", String::from_utf8_lossy(&out.stderr)), 
            pdf_url: None 
        })),
        Err(e) => Ok(Json(PrintResponse { 
            success: false, 
            message: e.to_string(), 
            pdf_url: None 
        })),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    dotenvy::dotenv().ok();
    
    let client_id = env::var("YANDEX_CLIENT_ID").expect("YANDEX_CLIENT_ID must be set");
    let client_secret = env::var("YANDEX_CLIENT_SECRET").expect("YANDEX_CLIENT_SECRET must be set");
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    fs::create_dir_all("data").ok();
    fs::create_dir_all("templates").ok();
    setup_typst_binary();

    let db_url = "sqlite:data/road_lists.db?mode=rwc";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Не удалось подключиться к SQLite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            yandex_id TEXT UNIQUE NOT NULL,
            email TEXT,
            name TEXT,
            avatar TEXT,
            created_at INTEGER NOT NULL
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS drivers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            driving_license TEXT,
            tractor_license TEXT,
            snils TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vehicles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            license_plate TEXT,
            sts TEXT,
            vehicle_type TEXT,
            category TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS company_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER UNIQUE NOT NULL,
            company_name TEXT,
            company_address TEXT,
            company_inn TEXT,
            dispatcher_name TEXT,
            mechanic_name TEXT,
            medic_name TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS default_values (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER UNIQUE NOT NULL,
            customer TEXT,
            loading_point TEXT,
            unloading_point TEXT,
            cargo TEXT,
            trips TEXT,
            distance TEXT,
            tons TEXT,
            arrival_time TEXT,
            field_object TEXT,
            field_area TEXT,
            field_norm TEXT,
            field_fact TEXT,
            field_motohours TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await.unwrap();

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://oauth.yandex.ru/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth.yandex.ru/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(format!("{}/auth/yandex/callback", base_url)).unwrap());

    let state = AppState {
        db: pool,
        oauth_client,
        http_client: reqwest::Client::new(),
    };

    let api_routes = Router::new()
        .route("/drivers", get(get_drivers).post(create_driver).put(update_driver).delete(delete_driver))
        .route("/vehicles", get(get_vehicles).post(create_vehicle).put(update_vehicle).delete(delete_vehicle))
        .route("/settings", get(get_settings).post(save_settings))
        .route("/defaults", get(get_defaults).post(save_defaults))
        .route("/print_batch", post(print_batch))
        .route("/me", get(get_current_user));

    let app = Router::new()
        .route("/auth/yandex/login", get(yandex_login))
        .route("/auth/yandex/callback", get(yandex_callback))
        .route("/auth/logout", get(logout))
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Сервер запущен на http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
