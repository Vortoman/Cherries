#[function_component(UserNamePrompt)]
fn input_prompt(app_state: &AppState) -> Html {
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();
    let on_cautious_change = {
        let input_value_handle = input_value_handle.clone();
        Callback::from(move |e: Event| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target: Option<EventTarget> = e.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                input_value_handle.set(input.value());
            }
        })
    };
    let submit_button: Callback<MouseEvent> = {
        let input_value = input_value.clone();
        Callback::from(move |_| {
            let input_value = input_value.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let new_user = User::new(input_value.clone());
                Request::post("/api/usernames/")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&new_user).unwrap())
                    .send()
                    .await
                    .unwrap();
            });
        })
    };
    html!(<>
          <label for="cautious-input">
                { "Enter user name:" }
                <input onchange={on_cautious_change}
                    id="user_name_input"
                    type="text"
                    value={input_value.clone()}
                />
            </label><button onclick={submit_button}>{"Submit"}</button>
        </>)
}
