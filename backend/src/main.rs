use actix_web::error::JsonPayloadError;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use common::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::sync::Mutex;

struct AppState {
    app_name: String,
    users: Mutex<Vec<User>>,
    universe: Mutex<Universe>,
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

#[post("/game/click")]
async fn handle_click(request: Json<Point>) -> Result<HttpResponse, JsonPayloadError> {
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

#[post("/files/back/")]
async fn get_from_frontend(request: Json<File>) -> Result<HttpResponse, JsonPayloadError> {
    let out = request.into_inner();
    println!("got data {} and {}", out.name, out.data);
    Ok(HttpResponse::Ok().body("success!"))
}

#[get("/files/{name}/")]
async fn download(info: web::Path<String>) -> HttpResponse {
    let name = String::from(info.into_inner().as_str());
    HttpResponse::Ok()
        .content_type("application/json")
        .body(name)
}

#[get("/usernames/delete")]
async fn get_deleted_input(request: Json<User>) -> Result<HttpResponse, JsonPayloadError> {
    let data = request.into_inner();
    println!("User {} left the game :(", data.name);
    Ok(HttpResponse::Ok().body(format!("{}, deleted", data.name)))
}

#[get("/images/pix")]
async fn serve_image() -> HttpResponse {
    let image_data = fs::read("./backend/data/pixelated.png").unwrap();
    HttpResponse::Ok()
        .content_type("image/png")
        .body(image_data)
}

#[post("/universe/cellpick")]
async fn cell_picked(
    app_state: Data<AppState>,
    request: Json<File>,
) -> Result<HttpResponse, JsonPayloadError> {
    Ok(HttpResponse::Ok().body("angekommen"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TRACE", "1");
    let app_state = web::Data::new(AppState {
        app_name: String::from("Actix Web"),
        users: Mutex::new(vec![]),
        universe: Mutex::new(Universe::new_rand()),
    });

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().app_data(app_state.clone()).service(
            web::scope("/api")
                .service(download)
                .service(serve_image)
                .service(get_from_frontend)
                .service(active_users)
                .service(register_user)
                .service(cell_picked),
        )
    })
    .bind(("127.0.0.2", 8080))?
    .run()
    .await
}
