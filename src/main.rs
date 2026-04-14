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
struct PrintRequest {
    driver_id: i32,
    vehicle_id: i32,
    tasks: Vec<TaskRow>,
    tractor_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchItem {
    driver_id: i32,
    vehicle_id: i32,
    date: Option<String>,
    tasks: Vec<TaskRow>,
    tractor_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PrintBatchRequest {
    items: Vec<PrintBatchItem>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct DefaultValues {
    id: i32,
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
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
struct CompanySettings {
    id: i32,
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
    vehicle_type: Option<String>,
    category: Option<String>
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
    sqlx::query("CREATE TABLE IF NOT EXISTS company_settings (id INTEGER PRIMARY KEY CHECK (id = 1), company_name TEXT, company_address TEXT, company_inn TEXT, dispatcher_name TEXT, mechanic_name TEXT, medic_name TEXT)")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS default_values (id INTEGER PRIMARY KEY CHECK (id = 1), customer TEXT, loading_point TEXT, unloading_point TEXT, cargo TEXT, trips TEXT, distance TEXT, tons TEXT, arrival_time TEXT, field_object TEXT, field_area TEXT, field_norm TEXT, field_fact TEXT, field_motohours TEXT)")
        .execute(&pool).await.unwrap();

    // Migrations (ignore errors if columns exist)
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN driving_license TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN tractor_license TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE drivers ADD COLUMN snils TEXT").execute(&pool).await;
    
    // Legacy pts rename to sts handled manually if needed, here we add new columns
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN pts TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN license_plate TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN sts TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN vehicle_type TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE vehicles ADD COLUMN category TEXT").execute(&pool).await;

    // Migrate any existing 'pts' data to 'sts'
    let _ = sqlx::query("UPDATE vehicles SET sts = pts WHERE sts IS NULL AND pts IS NOT NULL").execute(&pool).await;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drivers").fetch_one(&pool).await.unwrap();
    if count.0 == 0 {
        sqlx::query("INSERT INTO drivers (name) VALUES ('Иванов Иван Иванович'), ('Петров Петр Петрович')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO vehicles (name, vehicle_type) VALUES ('КамАЗ 65115', 'Грузовой'), ('Трактор МТЗ-82', 'Трактор')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO work_types (name) VALUES ('Перевозка зерна'), ('Вспашка поля')").execute(&pool).await.unwrap();
    }
    let settings_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM company_settings").fetch_one(&pool).await.unwrap();
    if settings_count.0 == 0 {
        sqlx::query("INSERT INTO company_settings (id) VALUES (1)").execute(&pool).await.unwrap();
    }
    let defaults_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM default_values").fetch_one(&pool).await.unwrap();
    if defaults_count.0 == 0 {
        sqlx::query("INSERT INTO default_values (id) VALUES (1)").execute(&pool).await.unwrap();
    }

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/api/drivers", get(get_drivers).post(create_driver).put(update_driver).delete(delete_driver))
        .route("/api/vehicles", get(get_vehicles).post(create_vehicle).put(update_vehicle).delete(delete_vehicle))
        .route("/api/works", get(get_works).post(create_work).put(update_work).delete(delete_work))
        .route("/api/print", post(print_waybill))
        .route("/api/print_batch", post(print_batch))
        .route("/api/settings", get(get_settings).post(save_settings))
        .route("/api/defaults", get(get_defaults).post(save_defaults))
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
struct CreateVehicleReq { name: String, license_plate: Option<String>, sts: Option<String>, vehicle_type: Option<String>, category: Option<String> }
#[derive(Deserialize)]
struct UpdateVehicleReq { id: i32, name: String, license_plate: Option<String>, sts: Option<String>, vehicle_type: Option<String>, category: Option<String> }

async fn create_vehicle(State(state): State<AppState>, Json(payload): Json<CreateVehicleReq>) -> Json<Vehicle> {
    let id = sqlx::query("INSERT INTO vehicles (name, license_plate, sts, vehicle_type, category) VALUES (?, ?, ?, ?, ?)")
        .bind(&payload.name).bind(&payload.license_plate).bind(&payload.sts).bind(&payload.vehicle_type).bind(&payload.category)
        .execute(&state.db).await.unwrap().last_insert_rowid() as i32;
    Json(Vehicle { id, name: payload.name, license_plate: payload.license_plate, sts: payload.sts, vehicle_type: payload.vehicle_type, category: payload.category })
}

async fn update_vehicle(State(state): State<AppState>, Json(payload): Json<UpdateVehicleReq>) -> Json<Vehicle> {
    sqlx::query("UPDATE vehicles SET name = ?, license_plate = ?, sts = ?, vehicle_type = ?, category = ? WHERE id = ?")
        .bind(&payload.name).bind(&payload.license_plate).bind(&payload.sts).bind(&payload.vehicle_type).bind(&payload.category).bind(payload.id)
        .execute(&state.db).await.unwrap();
    Json(Vehicle { id: payload.id, name: payload.name, license_plate: payload.license_plate, sts: payload.sts, vehicle_type: payload.vehicle_type, category: payload.category })
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

async fn get_settings(State(state): State<AppState>) -> Json<CompanySettings> {
    let settings: CompanySettings = sqlx::query_as("SELECT * FROM company_settings WHERE id = 1")
        .fetch_one(&state.db).await.unwrap();
    Json(settings)
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

async fn save_settings(State(state): State<AppState>, Json(payload): Json<SaveSettingsReq>) -> Json<CompanySettings> {
    sqlx::query("INSERT INTO company_settings (id, company_name, company_address, company_inn, dispatcher_name, mechanic_name, medic_name) VALUES (1, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET company_name=excluded.company_name, company_address=excluded.company_address, company_inn=excluded.company_inn, dispatcher_name=excluded.dispatcher_name, mechanic_name=excluded.mechanic_name, medic_name=excluded.medic_name")
        .bind(&payload.company_name)
        .bind(&payload.company_address)
        .bind(&payload.company_inn)
        .bind(&payload.dispatcher_name)
        .bind(&payload.mechanic_name)
        .bind(&payload.medic_name)
        .execute(&state.db).await.unwrap();
    Json(CompanySettings {
        id: 1,
        company_name: payload.company_name,
        company_address: payload.company_address,
        company_inn: payload.company_inn,
        dispatcher_name: payload.dispatcher_name,
        mechanic_name: payload.mechanic_name,
        medic_name: payload.medic_name,
    })
}

async fn get_defaults(State(state): State<AppState>) -> Json<DefaultValues> {
    let defaults: DefaultValues = sqlx::query_as("SELECT * FROM default_values WHERE id = 1")
        .fetch_one(&state.db).await.unwrap();
    Json(defaults)
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
}

async fn save_defaults(State(state): State<AppState>, Json(payload): Json<SaveDefaultsReq>) -> Json<DefaultValues> {
    sqlx::query("INSERT INTO default_values (id, customer, loading_point, unloading_point, cargo, trips, distance, tons, arrival_time, field_object, field_area, field_norm, field_fact, field_motohours) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET customer=excluded.customer, loading_point=excluded.loading_point, unloading_point=excluded.unloading_point, cargo=excluded.cargo, trips=excluded.trips, distance=excluded.distance, tons=excluded.tons, arrival_time=excluded.arrival_time, field_object=excluded.field_object, field_area=excluded.field_area, field_norm=excluded.field_norm, field_fact=excluded.field_fact, field_motohours=excluded.field_motohours")
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
        .execute(&state.db).await.unwrap();
    Json(DefaultValues {
        id: 1,
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
    })
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

async fn print_waybill(
    State(state): State<AppState>,
    Json(payload): Json<PrintRequest>
) -> Json<PrintResponse> {
    let driver: Driver = sqlx::query_as("SELECT * FROM drivers WHERE id = ?").bind(payload.driver_id).fetch_one(&state.db).await.unwrap();
    let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(payload.vehicle_id).fetch_one(&state.db).await.unwrap();
    let settings: CompanySettings = sqlx::query_as("SELECT * FROM company_settings WHERE id = 1").fetch_one(&state.db).await.unwrap();

    let template_base = fs::read_to_string("templates/template.typ").expect("Шаблон не найден");
    
    let drv_license = driver.driving_license.unwrap_or_default();
    let tr_license = driver.tractor_license.unwrap_or_default();
    let snils = driver.snils.unwrap_or_default();
    
    let license_plate = vehicle.license_plate.unwrap_or_default();
    let sts = vehicle.sts.unwrap_or_default();
    let vehicle_type = vehicle.vehicle_type.unwrap_or_else(|| "Грузовой".to_string());
    
    let typst_fn = if vehicle_type == "Трактор" { "#waybill_tractor" } else { "#waybill_truck" };
    
    let tractor_mode_val = payload.tractor_mode.unwrap_or_else(|| "cargo".to_string());
    let category = vehicle.category.unwrap_or_default();
    let call = format!(r#"
{}(
  driver: "{}",
  driving_license: "{}",
  tractor_license: "{}",
  snils: "{}",
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
  tasks: {},
  tractor_mode: "{}"
)
"#, 
        typst_fn,
        escape_typst(&driver.name),
        escape_typst(&drv_license),
        escape_typst(&tr_license),
        escape_typst(&snils),
        escape_typst(&vehicle.name),
        escape_typst(&license_plate),
        escape_typst(&sts),
        escape_typst(&category),
        escape_typst(&"09.04.2026"),
        escape_typst(&settings.company_name.unwrap_or_default()),
        escape_typst(&settings.company_address.unwrap_or_default()),
        escape_typst(&settings.company_inn.unwrap_or_default()),
        escape_typst(&settings.dispatcher_name.unwrap_or_default()),
        escape_typst(&settings.mechanic_name.unwrap_or_default()),
        escape_typst(&settings.medic_name.unwrap_or_default()),
        format_tasks_typst(&payload.tasks),
        escape_typst(&tractor_mode_val)
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
    let settings: CompanySettings = sqlx::query_as("SELECT * FROM company_settings WHERE id = 1").fetch_one(&state.db).await.unwrap();
    let mut compiled_typst = String::new();
    
    for (i, item) in payload.items.iter().enumerate() {
        let driver: Driver = sqlx::query_as("SELECT * FROM drivers WHERE id = ?").bind(item.driver_id).fetch_one(&state.db).await.unwrap();
        let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(item.vehicle_id).fetch_one(&state.db).await.unwrap();
        
        let drv_license = driver.driving_license.unwrap_or_default();
        let tr_license = driver.tractor_license.unwrap_or_default();
        let snils = driver.snils.unwrap_or_default();
        
        let license_plate = vehicle.license_plate.unwrap_or_default();
        let sts = vehicle.sts.unwrap_or_default();
        let vehicle_type = vehicle.vehicle_type.unwrap_or_else(|| "Грузовой".to_string());
        
        let date = item.date.clone().unwrap_or_else(|| "09.04.2026".to_string());
        let typst_fn = if vehicle_type == "Трактор" { "#waybill_tractor" } else { "#waybill_truck" };
        
        let tractor_mode_val = item.tractor_mode.clone().unwrap_or_else(|| "cargo".to_string());
        let category = vehicle.category.unwrap_or_default();
        let call = format!(r#"
{}(
  driver: "{}",
  driving_license: "{}",
  tractor_license: "{}",
  snils: "{}",
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
  tasks: {},
  tractor_mode: "{}"
)
"#, 
            typst_fn,
            escape_typst(&driver.name),
            escape_typst(&drv_license),
            escape_typst(&tr_license),
            escape_typst(&snils),
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
            format_tasks_typst(&item.tasks),
            escape_typst(&tractor_mode_val)
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
