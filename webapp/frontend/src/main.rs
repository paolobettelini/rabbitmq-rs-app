use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[at("/")]
    Home,
    #[at("/hello-server")]
    HelloServer,
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    yew::start_app::<App>;
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Route - Home" }</h1> },
        Route::HelloServer => html! { <HelloServer /> }
    }
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);

    // Request /api/hello once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/hello")
                        .send()
                        .await
                        .unwrap();
                    
                    let result = {
                        if !resp.ok() {
                            Err(format!(
                                "Error fetching data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.text().await.map_err(|e| e.to_string())
                        }
                    };
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{ "No server response" }</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div>{ "Got server response: "}{data}</div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{ "Error requesting data from server:" }{err}</div>
            }
        }
    }
}

// cargo install trunk
// rustup target add wasm32-unknown-unknown
// trunk serve --address 0.0.0.0 # for dev
// trunk build; cd ..; cargo run --bin webserver # for prod