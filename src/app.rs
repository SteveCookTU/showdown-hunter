use cfg_if::cfg_if;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use smog_strat_dex_rs::pokemon::MoveSet;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub fn register_server_functions() {
            _ = GetPokemon::register();
        }
    }
}

#[server(GetPokemon, "/api")]
pub async fn get_pokemon(cx: Scope, gen: String) -> Result<Vec<MoveSet>, ServerFnError> {
    use rand::Rng;
    use smog_strat_dex_rs::{basics::BasicsResponse, Client, Generation};
    use std::collections::{HashMap, HashSet};

    let generation = Generation::try_from(gen.as_str())
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let mut result = Vec::new();
    let mut added = HashSet::new();

    let mut rng = rand::thread_rng();

    let req = use_context::<actix_web::HttpRequest>(cx).ok_or(ServerFnError::ServerError(
        "Failed to get http request".to_string(),
    ))?;
    let basics = req.app_data::<actix_web::web::Data<HashMap<u8, BasicsResponse>>>();
    if let Some(basics) = basics {
        if let Some(basics) = basics.get(&(generation as u8)) {
            while added.len() < 6 {
                let pokemon_list = basics
                    .pokemon
                    .iter()
                    .filter(|p| p.is_non_standard.as_str() == "Standard")
                    .collect::<Vec<_>>();
                let index = rng.gen_range(0..pokemon_list.len());
                let basic_pokemon = pokemon_list[index];
                let name = basic_pokemon
                    .name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace("-mega", "");
                if added.contains(&name) {
                    continue;
                }
                if let Ok(pokemon) = Client::get_pokemon(generation, &name).await {
                    if pokemon.strategies.is_empty() {
                        continue;
                    }

                    let strat_index = rng.gen_range(0..pokemon.strategies.len());
                    let strat = pokemon.strategies.into_iter().nth(strat_index).unwrap();

                    if strat.move_sets.is_empty() {
                        continue;
                    }

                    let move_set_index = rng.gen_range(0..strat.move_sets.len());
                    added.insert(name);
                    result.push(strat.move_sets.into_iter().nth(move_set_index).unwrap());
                }
            }
        }
    }

    Ok(result)
}

#[component]
pub fn app(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    let (get_args, set_args) = create_signal(cx, ("sv".to_string(), 0));

    let pokemon = create_resource(cx, get_args, move |(gen, _)| get_pokemon(cx, gen));

    let pokemon_view = move || {
        match pokemon.read() {
            Some(resp) => {
                resp.iter().flatten().map(|p: &MoveSet| {
                    view!{
                        cx,
                        <div class="my-6">{
                            p.to_string().split('\n').map(|s| {
                                view! {
                                    cx,
                                    <p>{s.to_string()}</p>
                                }
                            }).collect::<Vec<_>>().into_view(cx)
                        }</div>
                    }
                }).collect::<Vec<_>>().into_view(cx)
            }
            None => {
                view! {
                    cx,
                    <div><p>"Click Get Pokemon"</p></div>
                }.into_view(cx)
            }
        }
    };

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/showdown-hunter.css"/>
        <Router>
            <Routes>
                <Route path="" view=move |cx| view! {
                    cx,
                    <main class="my-0 mx-auto max-w-3xl text-center text-sm">
                        <button on:click=move |_| set_args.set(("sv".to_string(), get_args.get().1 + 1))>"Get Pokemon"</button>
                        <Suspense fallback=move || view! {cx, <div>"Loading..."</div>}>
                            {pokemon_view}
                        </Suspense>
                    </main>
                }/>
            </Routes>
        </Router>
    }
}
