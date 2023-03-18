use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::error::JsonPayloadError;
// use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use actix_web_lab::web::spa;
use common::*;
use rand;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time;

const USER_NAME: &str = "user_name";
const USER_COLOR: &str = "user_color";
const USER_UNIVERSE_IDX: &str = "universe_index";

trait GenerateRandom {
    fn new_rand() -> Self;
}

impl GenerateRandom for Universe {
    fn new_rand() -> Universe {
        let mut cells = Vec::from([Cell::Empty; (WIDTH_UNIVERSE * HEIGHT_UNIVERSE) as usize]);
        let mut n_empty = WIDTH_UNIVERSE * HEIGHT_UNIVERSE;
        let mut n_neutral = 0;
        for _i in 0..N_NEUTRAL_BLOCKS {
            let idx = rand::random::<usize>() % cells.len();
            // let idx = ((((i as usize + 153) * 4) + 44) * 22) % uni.cells.len();
            if cells[idx] == Cell::Empty {
                cells[idx] = Cell::Neutral;
                n_empty -= 1;
                n_neutral += 1;
            }
        }
        Universe::_new_rand(cells, n_empty, n_neutral)
    }
}

struct AppState {
    app_name: String,
    users: Mutex<Vec<User>>,
    universe: Mutex<HashMap<u32, Universe>>,
    active_user: Mutex<ActiveUser>,
    uni_id: AtomicU32,
}

struct ActiveUser {
    time: time::Instant,
    pub status: Color,
    pub cell_picked: bool,
}

impl ActiveUser {
    fn new() -> Self {
        Self {
            time: time::Instant::now(),
            status: Color::Red,
            cell_picked: false,
        }
    }
}

// async fn get_user_data(session: Session, app_state: Data<AppState>) -> Result<String, Error> {
//     let name = session.get::<String>(USER_NAME)?;
//
//     Ok(name.unwrap())
// }

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

#[post("/usernames/")]
async fn register_user(
    session: Session,
    app_state: Data<AppState>,
    request: Json<User>,
) -> Result<HttpResponse, JsonPayloadError> {
    let mut users = app_state
        .users
        .lock()
        .map_err(|_| JsonPayloadError::ContentType)?;
    let new_user: User = request.into_inner();

    match session.get::<String>(USER_NAME) {
        Err(_) | Ok(None) => {
            users.push(new_user.clone());
            let _ = session.insert(USER_NAME, new_user.name);
        }
        Ok(Some(_)) => {
            println!("User bereits vorhanden.");
        }
    }
    println!("{:?}", users);
    Ok(HttpResponse::Ok().body("success creating new user"))
}

#[get("/user/color")]
async fn give_user_color(app_state: Data<AppState>, session: Session) -> HttpResponse {
    let color;
    let uidx = &app_state.uni_id;
    let early_return = {
        HttpResponse::Ok().content_type("application/json").body(
            serde_json::to_string(&ColorSender {
                value: "none".into(),
            })
            .unwrap(),
        )
    };
    let mut universe_handle = match app_state.universe.lock() {
        Ok(v) => v,
        Err(_) => return early_return,
    };
    let mut uni = match universe_handle.get_mut(&uidx.load(Ordering::SeqCst)) {
        Some(v) => v,
        None => return early_return,
    };
    if !uni.red_player_connected {
        color = "red";
        uni.red_player_connected = true;
    } else if !uni.blue_player_connected {
        color = "blue";
        uni.blue_player_connected = true;
    } else {
        color = "red";
        uidx.fetch_add(1, Ordering::SeqCst);
        let mut uni = Universe::new_rand();
        uni.red_player_connected = true;
        universe_handle.insert(uidx.load(Ordering::SeqCst), uni);
    }
    let _ = session.insert(USER_COLOR, color);
    let _ = session.insert(USER_UNIVERSE_IDX, uidx);
    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&ColorSender {
            value: color.into(),
        })
        .unwrap(),
    )
}

#[get("/usernames/total/")]
async fn active_users(app_state: Data<AppState>) -> HttpResponse {
    let users = match app_state.users.lock() {
        Ok(v) => v,
        Err(_) => return HttpResponse::Ok().body("Error. Could not get users"),
    };
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
async fn serve_universe(app_state: Data<AppState>, session: Session) -> HttpResponse {
    let uidx = session.get::<usize>(USER_UNIVERSE_IDX).unwrap().unwrap() as u32;
    let mut universe_handle = app_state.universe.lock().unwrap();
    let uni = universe_handle.get_mut(&uidx).unwrap();

    let mut au = app_state.active_user.lock().unwrap();

    if au.time.elapsed().as_secs_f64() >= 2.0 {
        println!("evolve");
        uni.evolve();
        au.time = time::Instant::now();
        au.cell_picked = false;
        if au.status == Color::Red {
            au.status = Color::Blue
        } else if au.status == Color::Blue {
            au.status = Color::Red
        }
    }

    uni.set_timer(au.time.elapsed().as_secs_f64());
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&uni.clone()).unwrap())
}

#[post("/universe/kill")]
async fn kill_universe(
    session: Session,
    app_state: Data<AppState>,
) -> Result<HttpResponse, JsonPayloadError> {
    let name = session.remove(USER_NAME).unwrap();
    session.remove(USER_COLOR);
    session.remove(USER_UNIVERSE_IDX);
    let mut users = app_state.users.lock().unwrap();
    let uidx = &app_state.uni_id;
    let mut universes = app_state.universe.lock().unwrap();
    universes.remove(&uidx.load(Ordering::SeqCst));
    for i in 0..users.len() {
        if users[i].name == name {
            users.remove(i);
            break;
        }
    }
    Ok(HttpResponse::Ok().body("Universe deleted"))
}

#[post("/universe/cellpick")]
async fn cell_picked(
    session: Session,
    app_state: Data<AppState>,
    request: Json<(f64, f64)>,
) -> Result<HttpResponse, JsonPayloadError> {
    let c = session.get::<String>(USER_COLOR).unwrap().unwrap();
    let uidx = session.get::<usize>(USER_UNIVERSE_IDX).unwrap().unwrap() as u32;
    let cell = if c == "red" {
        Cell::Red
    } else if c == "blue" {
        Cell::Blue
    } else {
        unreachable!()
    };
    let mut au = app_state.active_user.lock().unwrap();
    if au.status == cell.to_color() && au.cell_picked == false {
        au.cell_picked = true;
        drop(au);

        let coords = request.into_inner();
        let coords = (coords.1 as usize, coords.0 as usize);
        let mut universe = app_state.universe.lock().unwrap();
        let _ = universe
            .get_mut(&uidx)
            .unwrap()
            .set_cell(&cell, coords)
            .unwrap();
    }
    Ok(HttpResponse::Ok().body("angekommen"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TRACE", "1");
    let app_state = web::Data::new(AppState {
        app_name: String::from("Actix Web"),
        users: Mutex::new(vec![]),
        universe: Mutex::new(HashMap::from([(0, Universe::new_rand())])),
        active_user: Mutex::new(ActiveUser::new()),
        uni_id: AtomicU32::new(0),
    });

    let secret_key = Key::generate();
    HttpServer::new(move || {
        // let logger = Logger::default();
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .service(active_users)
                    .service(register_user)
                    .service(cell_picked)
                    .service(serve_universe)
                    .service(give_user_color),
            )
            .service(
                spa()
                    .index_file("./dist/index.html")
                    .static_resources_mount("/")
                    .static_resources_location("./dist")
                    .finish(),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
