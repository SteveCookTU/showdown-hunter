use crate::poll::*;
use leptos::*;
use leptos_meta::*;
use smog_strat_dex_rs::pokemon::MoveSet;

#[server(GetPokemon, "/api")]
pub async fn get_pokemon(
    cx: Scope,
    gen: String,
    count: usize,
) -> Result<Vec<MoveSet>, ServerFnError> {
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
            while added.len() < count {
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

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/showdown-hunter.css"/>
        <Main />
    }
}

#[component]
fn main(cx: Scope) -> impl IntoView {
    view! { cx,
        <main class="bg-slate-400 my-0 text-center text-sm h-screen">
            <Poll />
        </main>
    }
}
