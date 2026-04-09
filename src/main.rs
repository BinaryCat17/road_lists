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
use std::path::{PathBuf};
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

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchItem {
    driver_id: i32,
    vehicle_id: i32,
    work_type_id: i32,
    date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchRequest {
    items: Vec<PrintBatchItem>,
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
    name: String, 
    driving_license: Option<String>, 
    tractor_license: Option<String>,
    snils: Option<String>
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct Vehicle { 
    id: i32, 
    name: String, 
    license_plate: Option<String>,
    sts: Option<String>,
    vehicle_type: Option<String>
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct WorkType { id: i32, name: String }

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
        "CREATE TABLE IF NOT EXISTS drivers (id INTEGER PRIMARY KEY, name TEXT);
         CREATE TABLE IF NOT EXISTS vehicles (id INTEGER PRIMARY KEY, name TEXT);
         CREATE TABLE IF NOT EXISTS work_types (id INTEGER PRIMARY KEY, name TEXT);"
    ).execute(&pool).await.unwrap();

    // Migrations (ignore errors if columns exist)
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN driving_license TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN tractor_license TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN snils TEXT").execute(&pool).await;
    
    // Legacy pts rename to sts handled manually if needed, here we add new columns
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN pts TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN license_plate TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN sts TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN vehicle_type TEXT").execute(&pool).await;

    // Migrate any existing 'pts' data to 'sts'
    let _ = sqlx::query("UPDATE vehicles SET sts = pts WHERE sts IS NULL AND pts IS NOT NULL").execute(&pool).await;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drivers").fetch_one(&pool).await.unwrap();
    if count.0 == 0 {
        sqlx::query("INSERT INTO drivers (name) VALUES ('Иванов Иван Иванович'), ('Петров Петр Петрович')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO vehicles (name, vehicle_type) VALUES ('КамАЗ 65115', 'Грузовой'), ('Трактор МТЗ-82', 'Трактор')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO work_types (name) VALUES ('Перевозка зерна'), ('Вспашка поля')").execute(&pool).await.unwrap();
    }

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/api/drivers", get(get_drivers).post(create_driver).put(update_driver).delete(delete_driver))
        .route("/api/vehicles", get(get_vehicles).post(create_vehicle).put(update_vehicle).delete(delete_vehicle))
        .route("/api/works", get(get_works).post(create_work).put(update_work).delete(delete_work))
        .route("/api/print", post(print_waybill))
        .route("/api/print_batch", post(print_batch))
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

#[derive(Deserialize)]
struct CreateDriverReq { name: String, driving_license: Option<String>, tractor_license: Option<String>, snils: Option<String> }
#[derive(Deserialize)]
struct UpdateDriverReq { id: i32, name: String, driving_license: Option<String>, tractor_license: Option<String>, snils: Option<String> }
#[derive(Deserialize)]
struct DeleteReq { id: i32 }

async fn create_driver(State(state): State<AppState>, Json(payload): Json<CreateDriverReq>) -> Json<Driver> {
    let id = sqlx::query("INSERT INTO drivers (name, driving_license, tractor_license, snils) VALUES (?, ?, ?, ?)")
        .bind(&payload.name).bind(&payload.driving_license).bind(&payload.tractor_license).bind(&payload.snils)
        .execute(&state.db).await.unwrap().last_insert_rowid() as i32;
    Json(Driver { id, name: payload.name, driving_license: payload.driving_license, tractor_license: payload.tractor_license, snils: payload.snils })
}

async fn update_driver(State(state): State<AppState>, Json(payload): Json<UpdateDriverReq>) -> Json<Driver> {
    sqlx::query("UPDATE drivers SET name = ?, driving_license = ?, tractor_license = ?, snils = ? WHERE id = ?")
        .bind(&payload.name).bind(&payload.driving_license).bind(&payload.tractor_license).bind(&payload.snils).bind(payload.id)
        .execute(&state.db).await.unwrap();
    Json(Driver { id: payload.id, name: payload.name, driving_license: payload.driving_license, tractor_license: payload.tractor_license, snils: payload.snils })
}

async fn delete_driver(State(state): State<AppState>, Json(payload): Json<DeleteReq>) -> Json<()> {
    sqlx::query("DELETE FROM drivers WHERE id = ?").bind(payload.id).execute(&state.db).await.unwrap();
    Json(())
}

async fn get_vehicles(State(state): State<AppState>) -> Json<Vec<Vehicle>> {
    let vehicles = sqlx::query_as::<_, Vehicle>("SELECT * FROM vehicles").fetch_all(&state.db).await.unwrap();
    Json(vehicles)
}

#[derive(Deserialize)]
struct CreateVehicleReq { name: String, license_plate: Option<String>, sts: Option<String>, vehicle_type: Option<String> }
#[derive(Deserialize)]
struct UpdateVehicleReq { id: i32, name: String, license_plate: Option<String>, sts: Option<String>, vehicle_type: Option<String> }

async fn create_vehicle(State(state): State<AppState>, Json(payload): Json<CreateVehicleReq>) -> Json<Vehicle> {
    let id = sqlx::query("INSERT INTO vehicles (name, license_plate, sts, vehicle_type) VALUES (?, ?, ?, ?)")
        .bind(&payload.name).bind(&payload.license_plate).bind(&payload.sts).bind(&payload.vehicle_type)
        .execute(&state.db).await.unwrap().last_insert_rowid() as i32;
    Json(Vehicle { id, name: payload.name, license_plate: payload.license_plate, sts: payload.sts, vehicle_type: payload.vehicle_type })
}

async fn update_vehicle(State(state): State<AppState>, Json(payload): Json<UpdateVehicleReq>) -> Json<Vehicle> {
    sqlx::query("UPDATE vehicles SET name = ?, license_plate = ?, sts = ?, vehicle_type = ? WHERE id = ?")
        .bind(&payload.name).bind(&payload.license_plate).bind(&payload.sts).bind(&payload.vehicle_type).bind(payload.id)
        .execute(&state.db).await.unwrap();
    Json(Vehicle { id: payload.id, name: payload.name, license_plate: payload.license_plate, sts: payload.sts, vehicle_type: payload.vehicle_type })
}

async fn delete_vehicle(State(state): State<AppState>, Json(payload): Json<DeleteReq>) -> Json<()> {
    sqlx::query("DELETE FROM vehicles WHERE id = ?").bind(payload.id).execute(&state.db).await.unwrap();
    Json(())
}

async fn get_works(State(state): State<AppState>) -> Json<Vec<WorkType>> {
    let works = sqlx::query_as::<_, WorkType>("SELECT * FROM work_types").fetch_all(&state.db).await.unwrap();
    Json(works)
}

#[derive(Deserialize)]
struct CreateWorkReq { name: String }
#[derive(Deserialize)]
struct UpdateWorkReq { id: i32, name: String }

async fn create_work(State(state): State<AppState>, Json(payload): Json<CreateWorkReq>) -> Json<WorkType> {
    let id = sqlx::query("INSERT INTO work_types (name) VALUES (?)")
        .bind(&payload.name)
        .execute(&state.db).await.unwrap().last_insert_rowid() as i32;
    Json(WorkType { id, name: payload.name })
}

async fn update_work(State(state): State<AppState>, Json(payload): Json<UpdateWorkReq>) -> Json<WorkType> {
    sqlx::query("UPDATE work_types SET name = ? WHERE id = ?")
        .bind(&payload.name).bind(payload.id)
        .execute(&state.db).await.unwrap();
    Json(WorkType { id: payload.id, name: payload.name })
}

async fn delete_work(State(state): State<AppState>, Json(payload): Json<DeleteReq>) -> Json<()> {
    sqlx::query("DELETE FROM work_types WHERE id = ?").bind(payload.id).execute(&state.db).await.unwrap();
    Json(())
}

async fn print_waybill(
    State(state): State<AppState>,
    Json(payload): Json<PrintRequest>
) -> Json<PrintResponse> {
    let driver: Driver = sqlx::query_as("SELECT * FROM drivers WHERE id = ?").bind(payload.driver_id).fetch_one(&state.db).await.unwrap();
    let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(payload.vehicle_id).fetch_one(&state.db).await.unwrap();
    let work: WorkType = sqlx::query_as("SELECT * FROM work_types WHERE id = ?").bind(payload.work_type_id).fetch_one(&state.db).await.unwrap();

    let template_base = fs::read_to_string("templates/template.typ").expect("Шаблон не найден");
    
    let drv_license = driver.driving_license.unwrap_or_default();
    let tr_license = driver.tractor_license.unwrap_or_default();
    let snils = driver.snils.unwrap_or_default();
    
    let license_plate = vehicle.license_plate.unwrap_or_default();
    let sts = vehicle.sts.unwrap_or_default();
    let vehicle_type = vehicle.vehicle_type.unwrap_or_else(|| "Грузовой".to_string());
    
    let typst_fn = if vehicle_type == "Трактор" { "#waybill_tractor" } else { "#waybill_truck" };
    
    let call = format!(r#"
{}(
  driver: "{}",
  driving_license: "{}",
  tractor_license: "{}",
  snils: "{}",
  vehicle: "{}",
  license_plate: "{}",
  sts: "{}",
  work: "{}",
  date: "{}"
)
"#, 
        typst_fn,
        driver.name.replace("\"", "\\\""),
        drv_license.replace("\"", "\\\""),
        tr_license.replace("\"", "\\\""),
        snils.replace("\"", "\\\""),
        vehicle.name.replace("\"", "\\\""),
        license_plate.replace("\"", "\\\""),
        sts.replace("\"", "\\\""),
        work.name.replace("\"", "\\\""),
        "09.04.2026"
    );

    let temp_typ = "data/temp.typ";
    fs::write(temp_typ, format!("{}\n{}", template_base, call)).unwrap();

    let typst_exe = get_exe_dir().join(if cfg!(windows) { "typst.exe" } else { "typst" });
    let output = Command::new(typst_exe).arg("compile").arg(temp_typ).arg("static/out.pdf").output();
    let _ = fs::remove_file(temp_typ);

    match output {
        Ok(out) if out.status.success() => Json(PrintResponse { success: true, message: "OK".to_string(), pdf_url: Some("/out.pdf".to_string()) }),
        Ok(out) => Json(PrintResponse { success: false, message: format!("Ошибка Typst (код {}): {}", out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stderr)), pdf_url: None }),
        Err(e) => Json(PrintResponse { success: false, message: e.to_string(), pdf_url: None }),
    }
}

async fn print_batch(State(state): State<AppState>, Json(payload): Json<PrintBatchRequest>) -> Json<PrintResponse> {
    let template_base = fs::read_to_string("templates/template.typ").expect("Шаблон не найден");
    let mut compiled_typst = String::new();
    
    for (i, item) in payload.items.iter().enumerate() {
        let driver: Driver = sqlx::query_as("SELECT * FROM drivers WHERE id = ?").bind(item.driver_id).fetch_one(&state.db).await.unwrap();
        let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(item.vehicle_id).fetch_one(&state.db).await.unwrap();
        let work: WorkType = sqlx::query_as("SELECT * FROM work_types WHERE id = ?").bind(item.work_type_id).fetch_one(&state.db).await.unwrap();
        
        let drv_license = driver.driving_license.unwrap_or_default();
        let tr_license = driver.tractor_license.unwrap_or_default();
        let snils = driver.snils.unwrap_or_default();
        
        let license_plate = vehicle.license_plate.unwrap_or_default();
        let sts = vehicle.sts.unwrap_or_default();
        let vehicle_type = vehicle.vehicle_type.unwrap_or_else(|| "Грузовой".to_string());
        
        let date = item.date.clone().unwrap_or_else(|| "09.04.2026".to_string());
        let typst_fn = if vehicle_type == "Трактор" { "#waybill_tractor" } else { "#waybill_truck" };
        
        let call = format!(r#"
{}(
  driver: "{}",
  driving_license: "{}",
  tractor_license: "{}",
  snils: "{}",
  vehicle: "{}",
  license_plate: "{}",
  sts: "{}",
  work: "{}",
  date: "{}"
)
"#, 
            typst_fn,
            driver.name.replace("\"", "\\\""),
            drv_license.replace("\"", "\\\""),
            tr_license.replace("\"", "\\\""),
            snils.replace("\"", "\\\""),
            vehicle.name.replace("\"", "\\\""),
            license_plate.replace("\"", "\\\""),
            sts.replace("\"", "\\\""),
            work.name.replace("\"", "\\\""),
            date.replace("\"", "\\\"")
        );
        
        if i > 0 {
            compiled_typst.push_str("\n#pagebreak()\n");
        }
        compiled_typst.push_str(&call);
    }
    
    let temp_typ = "data/temp_batch.typ";
    fs::write(temp_typ, format!("{}\n{}", template_base, compiled_typst)).unwrap();

    let typst_exe = get_exe_dir().join(if cfg!(windows) { "typst.exe" } else { "typst" });
    let output = Command::new(typst_exe).arg("compile").arg(temp_typ).arg("static/out_batch.pdf").output();
    let _ = fs::remove_file(temp_typ);

    match output {
        Ok(out) if out.status.success() => Json(PrintResponse { success: true, message: "OK".to_string(), pdf_url: Some("/out_batch.pdf".to_string()) }),
        Ok(out) => Json(PrintResponse { success: false, message: format!("Ошибка: {}", String::from_utf8_lossy(&out.stderr)), pdf_url: None }),
        Err(e) => Json(PrintResponse { success: false, message: e.to_string(), pdf_url: None }),
    }
}
