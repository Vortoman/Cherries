#[derive(Clone, Serialize, Deserialize, Debug)]
struct Image {
    id: u32,
    name: String,
    artist: String,
}
impl Image {
    fn new() -> Image {
        Image {
            id: 0,
            name: String::from(""),
            artist: String::from(""),
        }
    }
}

struct ImageButtonState {
    value: bool,
}

fn send_to_back() -> Request {
    let url = "/api/files/back/";
    let request = Request::post(url)
        .header("Content-Type", "application/json")
        .body(r#"{"name":"John Doe","data":"john.doe@example.com"}"#);
    request
}

#[function_component()]
fn ToggleImage() -> Html {
    let state = use_state(|| ImageButtonState { value: false });
    let button: Callback<MouseEvent> = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(match state.value {
                true => ImageButtonState { value: false },
                false => ImageButtonState { value: true },
            });
            wasm_bindgen_futures::spawn_local(async move {
                send_to_back().send().await.unwrap();
            })
        })
    };
    html! {
            <div>
            <button onclick={button}>{"gib mir ein Bild!"}</button>
            if state.value {
            <img src="/api/images/pix" width="400" />}
            </div>
    }
}

#[function_component()]
fn ServeHtml() -> Html {
    let json_data = use_state(|| Image::new());
    {
        let json_data = json_data.clone();
        use_effect(move || {
            let json_data = json_data.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = "/api/images/test/";
                let response: Image = Request::get(url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                console::log_1(&JsValue::from(serde_json::to_string(&response).unwrap()));
                json_data.set(response);
            });
            || ()
        });
    };
    let debug_str = format!("{:?}", json_data);
    html! {
            <div>
            <p>{debug_str}</p>
            </div>
    }
}
