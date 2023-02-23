use actix_web::error::JsonPayloadError;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use common::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Mutex;
use std::{fs, time};

struct AppState {
    app_name: String,
    users: Mutex<Vec<User>>,
    universe: Mutex<Universe>,
    active_user: Mutex<ActiveUser>,
}

struct ActiveUser {
    name: String,
    time: time::Instant,
    // users: Option<Box<&'static dyn Iterator<Item = User>>>,
}

impl ActiveUser {
    fn new() -> Self {
        Self {
            name: "".into(),
            time: time::Instant::now(),
            // users: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BackendJson {
    id: u32,
    name: String,
    artist: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    name: String,
    data: String,
}

#[derive(Deserialize)]
struct Download {
    name: String,
}

#[post("/usernames/")]
async fn register_user(
    app_state: Data<AppState>,
    request: Json<User>,
) -> Result<HttpResponse, JsonPayloadError> {
    let mut users = app_state.users.lock().unwrap();
    let new_user: User = request.into_inner();
    users.push(new_user);
    println!("{:?}", users);
    Ok(HttpResponse::Ok().body("success creating new user"))
}

#[get("/usernames/total/")]
async fn active_users(app_state: Data<AppState>) -> HttpResponse {
    let users = app_state.users.lock().unwrap();
    let n_users = users.len() as u32;
    let user_list = UserList {
        users: users.clone(),
        n_users,
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&user_list).unwrap())
}
#[get("/usernames/delete")]
async fn get_deleted_input(request: Json<User>) -> Result<HttpResponse, JsonPayloadError> {
    let data = request.into_inner();
    println!("User {} left the game :(", data.name);
    Ok(HttpResponse::Ok().body(format!("{}, deleted", data.name)))
}

#[get("/universe/universe")]
async fn serve_universe(app_state: Data<AppState>) -> HttpResponse {
    let mut uni = app_state.universe.lock().unwrap();

    let users = app_state.users.lock().unwrap();
    let mut au = app_state.active_user.lock().unwrap();

    if au.name == "" {
        au.time = time::Instant::now();
        au.name = users[0].name.clone();
        println!("init");
    } else if au.time.elapsed().as_secs_f64() >= 3.0 {
        println!("evolve");
        uni.evolve();
        au.time = time::Instant::now();
    }
    uni.set_timer(au.time.elapsed().as_secs_f64());

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&uni.clone()).unwrap())
}

#[post("/universe/cellpick")]
async fn cell_picked(
    app_state: Data<AppState>,
    request: Json<(f64, f64)>,
) -> Result<HttpResponse, JsonPayloadError> {
    let coords = request.into_inner();
    let coords = (coords.1 as usize, coords.0 as usize);
    let mut universe = app_state.universe.lock().unwrap();
    let _ = universe.set_cell(&Cell::Red, coords).unwrap();
    Ok(HttpResponse::Ok().body("angekommen"))
}

#[get("/universe/timer")]
async fn timer(app_state: Data<AppState>) -> HttpResponse {
    let users = app_state.users.lock().unwrap();
    let mut au = app_state.active_user.lock().unwrap();

    if au.name == "" {
        au.time = time::Instant::now();
        au.name = users[0].name.clone();
        println!("init");
    } else if au.time.elapsed().as_secs_f64() >= 3.0 {
        println!("evolve");
        let mut uni = app_state.universe.lock().unwrap();
        uni.evolve();
        au.time = time::Instant::now();
    }
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&users.clone()).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TRACE", "1");
    let app_state = web::Data::new(AppState {
        app_name: String::from("Actix Web"),
        users: Mutex::new(vec![]),
        universe: Mutex::new(Universe::new_rand()),
        active_user: Mutex::new(ActiveUser::new()),
    });
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().app_data(app_state.clone()).service(
            web::scope("/api")
                .service(active_users)
                .service(register_user)
                .service(cell_picked)
                .service(serve_universe)
                .service(timer),
        )
    })
    .bind(("127.0.0.2", 8080))?
    .run()
    .await
}
