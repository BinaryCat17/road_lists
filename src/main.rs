use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::process::Command;
use std::{fs};
use std::path::{Path, PathBuf};
use std::env;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintRequest {
    driver_id: i32,
    vehicle_id: i32,
    work_type_id: i32,
}

#[derive(Serialize)]
struct PrintResponse {
    success: bool,
    message: String,
    pdf_url: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Driver { id: i32, name: String }
#[derive(Serialize, sqlx::FromRow)]
struct Vehicle { id: i32, name: String }
#[derive(Serialize, sqlx::FromRow)]
struct WorkType { id: i32, name: String }

// Функция для получения пути к папке с исполняемым файлом
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
        println!("Копирование бинарника Typst в папку программы: {:?}", target_path);
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Создаем нужные папки, если их нет
    fs::create_dir_all("data").ok();
    fs::create_dir_all("templates").ok();
    
    // Подготовка бинарника Typst (копируем его к экзешнику)
    setup_typst_binary();

    // Путь к базе данных в папке data/
    let db_url = "sqlite:data/road_lists.db?mode=rwc";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Не удалось подключиться к SQLite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS drivers (id INTEGER PRIMARY KEY, name TEXT);
         CREATE TABLE IF NOT EXISTS vehicles (id INTEGER PRIMARY KEY, name TEXT);
         CREATE TABLE IF NOT EXISTS work_types (id INTEGER PRIMARY KEY, name TEXT);"
    ).execute(&pool).await.unwrap();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drivers").fetch_one(&pool).await.unwrap();
    if count.0 == 0 {
        sqlx::query("INSERT INTO drivers (name) VALUES ('Иванов Иван Иванович'), ('Петров Петр Петрович')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO vehicles (name) VALUES ('КамАЗ 65115 (У123АВ 77)'), ('Трактор МТЗ-82')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO work_types (name) VALUES ('Перевозка зерна'), ('Вспашка поля')").execute(&pool).await.unwrap();
    }

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/api/drivers", get(get_drivers))
        .route("/api/vehicles", get(get_vehicles))
        .route("/api/works", get(get_works))
        .route("/api/print", post(print_waybill))
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Сервер запущен на http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn get_drivers(State(state): State<AppState>) -> Json<Vec<Driver>> {
    let drivers = sqlx::query_as::<_, Driver>("SELECT * FROM drivers").fetch_all(&state.db).await.unwrap();
    Json(drivers)
}

async fn get_vehicles(State(state): State<AppState>) -> Json<Vec<Vehicle>> {
    let vehicles = sqlx::query_as::<_, Vehicle>("SELECT * FROM vehicles").fetch_all(&state.db).await.unwrap();
    Json(vehicles)
}

async fn get_works(State(state): State<AppState>) -> Json<Vec<WorkType>> {
    let works = sqlx::query_as::<_, WorkType>("SELECT * FROM work_types").fetch_all(&state.db).await.unwrap();
    Json(works)
}

async fn print_waybill(
    State(state): State<AppState>,
    Json(payload): Json<PrintRequest>
) -> Json<PrintResponse> {
    let driver: Driver = sqlx::query_as("SELECT * FROM drivers WHERE id = ?").bind(payload.driver_id).fetch_one(&state.db).await.unwrap();
    let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(payload.vehicle_id).fetch_one(&state.db).await.unwrap();
    let work: WorkType = sqlx::query_as("SELECT * FROM work_types WHERE id = ?").bind(payload.work_type_id).fetch_one(&state.db).await.unwrap();

    // Берем шаблон из папки templates/
    let mut template = fs::read_to_string("templates/template.typ").expect("Шаблон не найден");
    template = template.replace("{{driver}}", &driver.name);
    template = template.replace("{{vehicle}}", &vehicle.name);
    template = template.replace("{{work}}", &work.name);
    template = template.replace("{{date}}", "09.04.2026");

    // Временный файл в папке data/
    let temp_typ = "data/temp.typ";
    fs::write(temp_typ, template).unwrap();

    // Путь к бинарнику Typst рядом с экзешником
    let typst_exe = get_exe_dir().join(if cfg!(windows) { "typst.exe" } else { "typst" });

    let output = Command::new(typst_exe)
        .arg("compile")
        .arg(temp_typ)
        .arg("static/out.pdf")
        .output();

    // Удаляем временный файл сразу
    let _ = fs::remove_file(temp_typ);

    match output {
        Ok(out) if out.status.success() => {
            Json(PrintResponse {
                success: true,
                message: "PDF успешно создан".to_string(),
                pdf_url: Some("/out.pdf".to_string()),
            })
        },
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
            Json(PrintResponse {
                success: false,
                message: format!("Ошибка Typst (код {}): {}", out.status.code().unwrap_or(-1), err_msg),
                pdf_url: None,
            })
        },
        Err(e) => {
            Json(PrintResponse {
                success: false,
                message: format!("Не удалось запустить Typst: {}", e),
                pdf_url: None,
            })
        }
    }
}
