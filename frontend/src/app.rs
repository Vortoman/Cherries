use common::*;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::{JsCast, JsValue, __rt::IntoJsResult};
use wasm_bindgen_futures;
use web_sys::{
    console, CanvasRenderingContext2d, EventTarget, HtmlCanvasElement, HtmlInputElement,
};
use yew::prelude::*;
use yew_router::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

struct AppState {
    user_name: String,
}

impl AppState {
    fn new() -> Self {
        AppState {
            user_name: "".into(),
        }
    }
}

struct ImageButtonState {
    value: bool,
}

enum UserMsg {
    UpdateInput(String),
    ButtonPressed(String),
}

#[derive(Clone, PartialEq, Properties)]
struct UserProps {
    name: AttrValue,
    app_hook: Callback<AttrValue>,
}

struct UserNamePrompt {
    input_value: String,
}

impl Component for UserNamePrompt {
    type Message = UserMsg;
    type Properties = UserProps;

    fn create(_ctx: &Context<Self>) -> Self {
        UserNamePrompt {
            input_value: "".into(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            UserMsg::UpdateInput(val) => {
                self.input_value = val;
            }
            UserMsg::ButtonPressed(val) => ctx.props().app_hook.emit(val.clone().into()),
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let on_cautious_change = {
            let link = link.clone();
            Callback::from(move |e: Event| {
                let target = e.target();
                let input = target
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                    .unwrap()
                    .value();
                console::log_1(&JsValue::from(input.clone()));
                link.send_message(UserMsg::UpdateInput(input));
            })
        };
        let navigator = ctx.link().navigator().unwrap();
        let submit_button: Callback<MouseEvent> = {
            let input_value = self.input_value.clone();
            Callback::from(move |_| {
                let input_value = input_value.clone();
                link.send_message(UserMsg::ButtonPressed(input_value.clone()));
                let navigator = navigator.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let new_user = User::new(input_value.clone());
                    Request::post("/api/usernames/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&new_user).unwrap())
                        .send()
                        .await
                        .unwrap();
                    navigator.push(&Route::ActiveUsers);
                });
            })
        };
        html!(<>
          <label for="cautious-input">
                { "Enter user name:" }
                <input onchange={on_cautious_change}
                    id="user_name_input"
                    type="text"
                    value={self.input_value.clone()}
                />
            </label><button onclick={submit_button}>{"Submit"}</button>
        </>)
    }
}

#[derive(Clone, PartialEq, Properties)]
struct LobbyProperties {
    player_name: String,
}

enum LobbyMsg {
    UpdateUsers(UserList),
}

struct ActiveUsers {
    json_data: UserList,
}

impl Component for ActiveUsers {
    type Message = LobbyMsg;
    type Properties = LobbyProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            json_data: UserList::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LobbyMsg::UpdateUsers(val) => self.json_data = val,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let url = "/api/usernames/total/";
            let response = Request::get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            console::log_1(&JsValue::from(format!("{:?}", response)));
            link.send_message(LobbyMsg::UpdateUsers(response));
        });
        let n_users = self.json_data.n_users;
        let user_list = self.json_data.users.clone();
        let navigator = ctx.link().navigator().unwrap();
        let start_game: Callback<MouseEvent> =
            { Callback::from(move |_| navigator.push(&Route::InGame)) };
        html!(
        <>
            <p>{"Number of users "}{n_users}</p>
            <p>
                { for user_list.iter().map(|item| html!{ <li>{ item.name.clone() }</li> }) }
            </p>
            <p>{"your name: "}{ctx.props().player_name.clone()}</p>
            <button onclick={start_game}>{"start!!!"}</button>
        </>
        )
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        let url = "/api/usernames/delete";
        let props = ctx.props().clone();
        wasm_bindgen_futures::spawn_local(async move {
            Request::post(url)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&props.player_name.clone()).unwrap())
                .send()
                .await
                .unwrap();
        })
    }
}

pub struct InGame {
    canvas: NodeRef,
}

pub enum InGameMsg {
    Init,
    Render,
    CanvasClick(MouseEvent),
}

impl Component for InGame {
    type Message = InGameMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let canvas = NodeRef::default();
        ctx.link().send_message(InGameMsg::Init);
        InGame { canvas }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InGameMsg::Init => {
                self.init(ctx);
                true
            }
            InGameMsg::Render => {
                self.render();
                false
            }
            InGameMsg::CanvasClick(eve) => {
                let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
                let cctx: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();

                let x = eve.offset_x();
                let y = eve.offset_y();
                log!("clicko geilo {x}, {y}");

                let url = "/api/game/click";
                wasm_bindgen_futures::spawn_local(async move {
                    Request::post(url)
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&Point::new(x, y)).unwrap())
                        .send()
                        .await
                        .unwrap();
                });
                cctx.set_fill_style(&JsValue::from("rgb(0,79,92)"));
                cctx.begin_path();
                cctx.arc(x as f64, y as f64, 20.0, 0.0, 2.0 * std::f64::consts::PI)
                    .unwrap();
                cctx.fill();

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {<>
            <canvas id="drawing"
                width = {format!("{WIDTH_CANVAS}")}
            height = {format!("{HEIGHT_CANVAS}")}
                ref={self.canvas.clone()}
            onclick={ctx.link().callback(|event: web_sys::MouseEvent| InGameMsg::CanvasClick(event))}/>
                </>
        }
    }
}

impl InGame {
    fn init(&self, ctx: &Context<Self>) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let cctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        cctx.set_fill_style(&JsValue::from("white"));
        // cctx.set_fill_style(&JsValue::from("rgb(0,79,92)"));
        cctx.fill_rect(0.0, 0.0, WIDTH_CANVAS.into(), HEIGHT_CANVAS.into());
        // let _ = cctx.fill_text("hello", 200.0, 200.0);
        self.draw_grid();
        ctx.link().send_message(InGameMsg::Render);
    }
    fn render(&self) {}

    fn draw_grid(&self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let cctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        cctx.begin_path();
        cctx.set_fill_style(&JsValue::from(GRID_COLOR));

        // Vertical lines
        for i in 0..WIDTH_UNIVERSE {
            cctx.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0 as f64);
            cctx.line_to(
                (i * (CELL_SIZE + 1) + 1) as f64,
                ((CELL_SIZE + 1) * HEIGHT_UNIVERSE + 1) as f64,
            );
        }
        // Horizontal lines
        for j in 0..HEIGHT_UNIVERSE {
            cctx.move_to(0 as f64, (j * (CELL_SIZE + 1) + 1) as f64);
            cctx.line_to(
                ((CELL_SIZE + 1) * WIDTH_UNIVERSE + 1) as f64,
                (j * (CELL_SIZE + 1) + 1) as f64,
            );
        }
        cctx.stroke();
    }
}

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/lobby")]
    ActiveUsers,
    #[at("/game")]
    InGame,
}

pub enum AppMsg {
    UserName(AttrValue),
}

pub struct App {
    app_state: AppState,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            app_state: AppState::new(),
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::UserName(val) => {
                self.app_state.user_name = val.as_str().into();
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let app_hook = ctx.link().callback(AppMsg::UserName);
        let player_name = self.app_state.user_name.clone();
        let switch = move |routes: Route| -> Html {
            match routes {
                Route::Home => {
                    html! { <UserNamePrompt name="Peter" app_hook={app_hook.clone()} /> }
                }
                Route::ActiveUsers => html! {
                    <ActiveUsers player_name={player_name.clone()} />
                },
                Route::InGame => html! { <InGame/>},
            }
        };
        html!(
            <BrowserRouter>
                <Switch<Route> render = {switch} />
            </BrowserRouter>
        )
    }
}
