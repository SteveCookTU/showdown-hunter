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

    let on_click = create_action(cx, |_: &()| async { () });

    let pokemon = create_resource_with_initial_value(
        cx,
        move || (on_click.version().get()),
        move |_| get_pokemon(cx, "sv".to_string()),
        Some(Ok(vec![])),
    );

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/showdown-hunter.css"/>
        <Router>
            <main class="my-0 mx-auto max-w-3xl text-center text-sm">
                <button on:click=move |_| on_click.dispatch(())>"Get Pokemon"</button>
                {move || {
                    let p = pokemon.read().map(|res| match res {
                        Ok(pokemon) => {
                            if pokemon.is_empty() {
                                view! {
                                    cx,
                                    <div class="my-6"><p>"Click Get Pokemon"</p></div>
                                }.into_view(cx)
                            } else {
                                pokemon.iter().map(|p| {
                                    view! {
                                        cx,
                                        <div class="my-6">
                                            {
                                                p.to_string().split('\n').map(|s| {
                                                    view!{
                                                        cx,
                                                        <p>{s.to_string()}</p>
                                                    }
                                                }).collect::<Vec<_>>().into_view(cx)
                                            }
                                        </div>
                                    }
                                }).collect::<Vec<_>>().into_view(cx)
                            }
                        },
                        Err(e) => {
                            view! {
                                cx,
                                <div class="my-6"><p>{e.to_string()}</p></div>
                            }.into_view(cx)
                        }
                    }).unwrap_or_default();
                    view! {
                        cx,
                        {p}
                    }
                }}
            </main>
        </Router>
    }
}
