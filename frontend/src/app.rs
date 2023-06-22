use common::constants::*;
use common::*;
use reqwasm::http::Request;
use serde_json;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures;
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::{navigator, prelude::*};

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

enum UserMsg {
    UpdateInput(String),
    ButtonPressed(String),
}

#[derive(Clone, PartialEq, Properties)]
struct UserProps {
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
                log!("{}", input);
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

#[derive(PartialEq, Properties)]
struct VictoryProps {
    winner: String,
}

struct VictoryScreen;

impl Component for VictoryScreen {
    type Message = ();
    type Properties = VictoryProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let navigator = ctx.link().navigator().unwrap();
        let return_button: Callback<MouseEvent> = {
            Callback::from(move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    Request::post("/api/universe/kill").send().await.unwrap();
                });
                navigator.push(&Route::Home);
            })
        };
        html!(<>
                <p>{"the winner is: "}{props.winner.clone()}</p>
                <button onclick={return_button}>{"Back"}</button>
            </>)
    }
}

#[derive(Clone, PartialEq, Properties)]
struct LobbyProperties {
    player_name: String,
    app_hook: Callback<AttrValue>,
}

enum LobbyMsg {
    UpdateUsers(UserList),
    Color(String),
}

struct ActiveUsers {
    json_data: UserList,
}

impl Component for ActiveUsers {
    type Message = LobbyMsg;
    type Properties = LobbyProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            json_data: UserList::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LobbyMsg::UpdateUsers(val) => self.json_data = val,
            LobbyMsg::Color(val) => {
                let navigator = ctx.link().navigator().unwrap();
                if val != "none" {
                    navigator.push(&Route::InGame);
                }
            }
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
        let start_game: Callback<MouseEvent> = {
            let link = ctx.link().clone();
            Callback::from(move |_| {
                let link = link.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = "/api/user/color";
                    let response: ColorSender = Request::get(url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    link.send_message(LobbyMsg::Color(response.value))
                });
            })
        };
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
}

#[derive(Clone, PartialEq, Properties)]
pub struct PreGameProps {
    color: String,
}

pub enum PreGameMsg {
    CheckReady(bool),
}

pub struct PreGame;

impl Component for PreGame {
    type Message = PreGameMsg;
    type Properties = PreGameProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let color_string = ctx.props().color.clone();
        let navigator = ctx.link().navigator().unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let url = "/api/user/pregame";
            log!("sth");
            let response: PreGameData = Request::get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if response.ready {
                navigator.push(&Route::InGame)
            }
        });

        html!(
        <>
            <p>{"Your color is "}{color_string}</p>
            <p>{"waiting for second player"}</p>
        </>
        )
    }
}

pub struct InGame {
    canvas: NodeRef,
    universe: Universe,
}

#[derive(Clone, PartialEq, Properties)]
pub struct InGameProps {
    app_hook: Callback<AttrValue>,
}

pub enum InGameMsg {
    Init,
    Render(Universe),
    CanvasClick(MouseEvent),
}

impl Component for InGame {
    type Message = InGameMsg;
    type Properties = InGameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let canvas = NodeRef::default();
        ctx.link().send_message(InGameMsg::Init);
        InGame {
            canvas,
            universe: Universe::new_empty(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InGameMsg::Init => {
                self.init(ctx);
                true
            }
            InGameMsg::Render(uni) => {
                self.universe = uni;
                if self.universe.is_finished() {
                    let navigator = ctx.link().navigator().unwrap();
                    let cell_numbers = self.universe.get_cell_numbers();
                    let winner = if cell_numbers.1 > cell_numbers.2 {
                        "Red"
                    } else if cell_numbers.2 > cell_numbers.1 {
                        "Blue"
                    } else {
                        "None"
                    };
                    ctx.props().app_hook.emit(winner.into());
                    navigator.push(&Route::VictoryScreen);
                }
                self.render_universe();
                true
            }
            InGameMsg::CanvasClick(eve) => {
                let x = eve.offset_x() as u32;
                let y = eve.offset_y() as u32;
                log!("clicked at: {x}, {y}");

                let x_uni = x * WIDTH_UNIVERSE / WIDTH_CANVAS;
                let y_uni = y * HEIGHT_UNIVERSE / HEIGHT_CANVAS;
                log!("zelle:  {x_uni}, {y_uni}");
                let url = "/api/universe/cellpick";
                wasm_bindgen_futures::spawn_local(async move {
                    Request::post(url)
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&(x_uni, y_uni)).unwrap())
                        .send()
                        .await
                        .unwrap();
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let url = "/api/universe/universe";
            let response = Request::get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            console::log_1(&JsValue::from(format!("{:?}", response)));
            link.send_message(InGameMsg::Render(response));
        });
        let timer = self.universe.get_timer();
        let cell_numbers = self.universe.get_cell_numbers();
        html! {<>
            <canvas id="drawing"
                width = {format!("{WIDTH_CANVAS}")}
            height = {format!("{HEIGHT_CANVAS}")}
                ref={self.canvas.clone()}
            onclick={ctx.link().callback(|event: web_sys::MouseEvent| InGameMsg::CanvasClick(event))}/>
                <p>{"Timer: "}{format!("{:.2}", timer)}</p>
                <p>{"Empty Cells: "}{cell_numbers.0}</p>
                <p>{"Red Cells: "}{cell_numbers.1}</p>
                <p>{"Blue Cells: "}{cell_numbers.2}</p>
                <p>{"Neutral Cells: "}{cell_numbers.3}</p>
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
    }

    fn render_universe(&self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let cctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let cells = self.universe.get_cells();

        let fill_rectangles = |cell_style: Cell, cell_color: &str| {
            cctx.set_fill_style(&JsValue::from(cell_color));
            for row in 0..WIDTH_UNIVERSE {
                for col in 0..HEIGHT_UNIVERSE {
                    if cells[self
                        .universe
                        .get_index((row as usize, col as usize))
                        .unwrap()]
                        != cell_style
                    {
                        continue;
                    }
                    cctx.fill_rect(
                        (col * (CELL_SIZE + 1) + 1) as f64,
                        (row * (CELL_SIZE + 1) + 1) as f64,
                        CELL_SIZE as f64,
                        CELL_SIZE as f64,
                    );
                }
            }
        };

        cctx.begin_path();
        fill_rectangles(Cell::Empty, EMPTY_COLOR);
        fill_rectangles(Cell::Red, RED_COLOR);
        fill_rectangles(Cell::Blue, BLUE_COLOR);
        fill_rectangles(Cell::Neutral, WALL_COLOR);
        cctx.stroke();
    }

    fn fill_rects(&self, cells: &Vec<Cell>, cctx: &CanvasRenderingContext2d, cell_type: Cell) {
        for row in 0..WIDTH_UNIVERSE {
            for col in 0..HEIGHT_UNIVERSE {
                if cells[self
                    .universe
                    .get_index((row as usize, col as usize))
                    .unwrap()]
                    != cell_type
                {
                    continue;
                }
                cctx.fill_rect(
                    (col * (CELL_SIZE + 1) + 1) as f64,
                    (row * (CELL_SIZE + 1) + 1) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                );
            }
        }
    }

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

#[derive(Clone, PartialEq, Properties)]
struct VictoryScreenProps {
    winner: String,
}

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/lobby")]
    ActiveUsers,
    #[at("/pregame")]
    PreGame,
    #[at("/game")]
    InGame,
    #[at("/victory")]
    VictoryScreen,
}

pub enum AppMsg {
    UserName(AttrValue),
    InGame(AttrValue),
}

struct AppState {
    user_name: String,
    winner: String,
}

impl AppState {
    fn new() -> Self {
        AppState {
            user_name: "".into(),
            winner: "".into(),
        }
    }
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
            AppMsg::InGame(val) => {
                self.app_state.winner = val.as_str().into();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let app_hook = ctx.link().callback(AppMsg::UserName);
        let app_hook_game = ctx.link().callback(AppMsg::InGame);
        let player_name = self.app_state.user_name.clone();
        let winner_name = self.app_state.winner.clone();
        let switch = move |routes: Route| -> Html {
            match routes {
                Route::Home => {
                    html! { <UserNamePrompt app_hook={app_hook.clone()} /> }
                }
                Route::ActiveUsers => html! {
                    <ActiveUsers player_name={player_name.clone()} app_hook = {app_hook_au.clone()} />
                },
                Route::InGame => html! { <InGame app_hook={app_hook_game.clone()}/>},
                Route::VictoryScreen => {
                    html! { <VictoryScreen winner={winner_name.clone()}/> }
                }
            }
        };
        html!(
            <BrowserRouter>
                <Switch<Route> render = {switch} />
            </BrowserRouter>
        )
    }
}
