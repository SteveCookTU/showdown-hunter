use crate::pokemon::*;
use cfg_if::cfg_if;
use leptos::*;

#[cfg(feature = "hydrate")]
use crate::set_callback;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use broadcaster::BroadcastChannel;
        use std::sync::RwLock;
        use std::collections::HashMap;

        lazy_static::lazy_static! {
            pub static ref CURRENT_POLL_CHANNEL: BroadcastChannel<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>)> = BroadcastChannel::new();
            pub static ref POLL_RESULT_CHANNEL: BroadcastChannel<Option<smog_strat_dex_rs::pokemon::MoveSet>> = BroadcastChannel::new();
            static ref POLL_VOTES: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
        }

        static CURRENT_POLL: RwLock<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>)> = RwLock::new((false, vec![]));
        static POLL_RESULT: RwLock<Option<smog_strat_dex_rs::pokemon::MoveSet>> = RwLock::new(None);
        // This is a fake ID given by Truffle auth when visiting the page locally
        static STREAMERS: [&str; 1] = ["0235a210-b594-11ed-89d0-88467ddc6899"];
    }
}

#[server(GetIsStreamer, "/api")]
pub async fn get_is_streamer(uuid: String) -> Result<bool, ServerFnError> {
    Ok(STREAMERS.contains(&uuid.to_string().as_str()))
}

#[server(GetCurrentPoll, "/api")]
pub async fn get_current_poll(
) -> Result<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>), ServerFnError> {
    Ok((*CURRENT_POLL.read().unwrap()).clone())
}

#[server(GetPollResult, "/api")]
pub async fn get_poll_result() -> Result<Option<smog_strat_dex_rs::pokemon::MoveSet>, ServerFnError>
{
    Ok((*POLL_RESULT.read().unwrap()).clone())
}

#[server(SetCurrentVote, "/api")]
pub async fn set_current_vote(user_id: String, idx: usize) -> Result<(), ServerFnError> {
    POLL_VOTES.write().unwrap().insert(user_id, idx);
    Ok(())
}

#[server(GetCurrentVote, "/api")]
pub async fn get_current_vote(uuid: String) -> Result<Option<usize>, ServerFnError> {
    Ok(POLL_VOTES.read().unwrap().get(&uuid).copied())
}

#[component]
pub fn poll(cx: Scope) -> impl IntoView {
    let (get_user_id, _set_user_id) = create_signal(cx, String::new());
    let set_current_vote = create_server_action::<SetCurrentVote>(cx);
    let is_streamer = create_resource(cx, get_user_id, move |s: String| get_is_streamer(s));
    let current_vote = create_resource(cx, get_user_id, move |s| get_current_vote(s));
    let (open, set_open) = create_signal(cx, false);

    #[cfg(feature = "hydrate")]
    let closure = Closure::new(move |s| {
        _set_user_id.set(s);
    });

    #[cfg(feature = "hydrate")]
    create_effect(cx, move |_| {
        set_callback(&closure);
    });

    #[cfg(feature = "hydrate")]
    let poll = {
        use futures::StreamExt;
        let mut source = gloo_net::eventsource::futures::EventSource::new("/api/poll")
            .expect("Failed to connect to SSE stream");
        let s = create_signal_from_stream(
            cx,
            source
                .subscribe("poll")
                .expect("Failed to subscribe to poll event")
                .map(|val| {
                    serde_json::from_str::<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>)>(
                        &val.expect("no poll event")
                            .1
                            .data()
                            .as_string()
                            .expect("expected string"),
                    )
                    .expect("Failed to parse poll")
                }),
        );

        on_cleanup(cx, move || source.close());
        s
    };

    #[cfg(feature = "ssr")]
    let (poll, _) = create_signal(cx, None::<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>)>);

    #[cfg(feature = "hydrate")]
    let result = {
        use futures::StreamExt;
        let mut source = gloo_net::eventsource::futures::EventSource::new("/api/result")
            .expect("Failed to connect to SSE stream");
        let s = create_signal_from_stream(
            cx,
            source
                .subscribe("result")
                .expect("Failed to subscribe to poll event")
                .map(|val| {
                    serde_json::from_str::<Option<smog_strat_dex_rs::pokemon::MoveSet>>(
                        &val.expect("no result event")
                            .1
                            .data()
                            .as_string()
                            .expect("expected string"),
                    )
                    .expect("Failed to parse poll")
                }),
        );

        on_cleanup(cx, move || source.close());
        s
    };

    #[cfg(feature = "ssr")]
    let (result, _) = create_signal(cx, None::<Option<smog_strat_dex_rs::pokemon::MoveSet>>);

    let is_streamer_view = move || {
        is_streamer.read(cx).map(|res| {
            res.iter()
                .map(|b| {
                    if *b {
                        view! {
                            cx,
                            <StreamerButtons user_id={get_user_id.get()}/>
                        }
                        .into_view(cx)
                    } else {
                        view! {
                            cx,
                            <></>
                        }
                        .into_view(cx)
                    }
                })
                .collect::<Vec<_>>()
                .into_view(cx)
        })
    };

    let options_view = move || {
        let current_vote = current_vote
            .read(cx)
            .map(|res| res.unwrap())
            .unwrap_or_default();

        let poll_view = poll.with(|poll: &Option<(bool, Vec<smog_strat_dex_rs::pokemon::MoveSet>)>| {
            if let Some(poll) = poll {
                let running = poll.0;
                if !running {
                    view! {
                        cx,
                        <div class="my-4 text-2xl font-semibold">"No Poll Currently Running"</div>
                    }.into_view(cx)
                } else {
                    view! {
                        cx,
                        <div class="grid grid-cols-2 grid-flow-row auto-rows-fr gap-4 place-self-center mx-4 mb-4">
                        {
                            #[cfg(feature = "hydrate")]
                            let len = poll.1.len();
                            let on_click = move |id: usize| {
                                if running { set_current_vote.dispatch(SetCurrentVote { user_id: get_user_id.get(), idx: id }); }
                                #[cfg(feature = "hydrate")]
                                {
                                    for j in 0..len {
                                        if let Some(window) = web_sys::window() {
                                            if let Some(document) = window.document() {
                                                if let Some(element) = document.get_element_by_id(&format!("poke-{}", j)) {
                                                    if j == id {
                                                        element.set_class_name("flex justify-center p-3 border-2 border-green-500 bg-slate-300 rounded-lg shadow-xl")
                                                    } else {
                                                        element.set_class_name("flex justify-center p-3 border-2 border-slate-700 rounded-lg shadow-xl")
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            };
                            poll.1.iter().enumerate().map(|(i, set)| {
                                if let Some(current) = current_vote {
                                    if current == i {
                                        return view! {
                                                cx,
                                                <div id={format!("poke-{}", i)} class="flex justify-center p-3 border-2 border-green-500 bg-slate-300 rounded-lg shadow-xl" on:click=move |_| on_click(i)>
                                                    <Pokemon name=set.pokemon.clone() set=set.to_string()/>
                                                </div>
                                            };
                                    }
                                }
                                view! {
                                    cx,
                                    <div id={format!("poke-{}", i)} class="flex justify-center p-3 border-2 border-slate-700 rounded-lg shadow-xl" on:click=move |_| on_click(i)>
                                        <Pokemon name=set.pokemon.clone() set=set.to_string()/>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_view(cx)
                        }
                        </div>
                    }.into_view(cx)
                }
            } else {
                view! {
                    cx,
                    <div class="text-3xl font-semibold">"No Poll Currently Running"</div>
                }.into_view(cx)
            }
        });

        let results_view = result.with(
            move |res: &Option<Option<smog_strat_dex_rs::pokemon::MoveSet>>| {
                if let Some(res) = res {
                    if let Some(res) = res {
                        return view! {
                            cx,
                            <div class="text-2xl underline font-semibold">"Previous Poll"</div>
                            <div class="my-6">
                                <Pokemon name=res.pokemon.clone() set=res.to_string()/>
                            </div>
                        }
                        .into_view(cx);
                    }
                }
                view! {
                    cx,
                    <></>
                }
                .into_view(cx)
            },
        );

        view! {
            cx,
            {poll_view}
            {results_view}
        }
    };

    view! {
        cx,
        {move || {
            if open.get() {
                view!{
                    cx,
                    <button class="px-3 my-2 bg-slate-300 rounded-md" on:click=move |_| {crate::minimize(); set_open.set(false);}>"Minimize"</button>
                    <div class="flex flex-col">
                        <Transition fallback=move || view!{cx, <div>"Loading"</div>}>
                            <div class="flex flex-row justify-evenly mb-2">
                                {is_streamer_view}
                            </div>
                        </Transition>
                        <div class="flex flex-col justify-center">
                            <Transition fallback=move || view!{cx, <div>"Loading"</div>}>
                                {options_view}
                            </Transition>
                        </div>
                    </div>
                }.into_view(cx)
            } else {
                view!{
                    cx,
                    <div style="height: 5px;"></div>
                    <div class="mx-1" style="height: 30px; background-repeat: no-repeat; background-size: contain; background-position: center center; background-image: url(\"/logo.png\")" on:click=move |_| {set_open.set(true); crate::maximize();}></div>
                    <div style="height: 5px;"></div>
                }.into_view(cx)
            }
        }}


    }
}

#[server(GeneratePoll, "/api")]
pub async fn generate_poll(cx: Scope, user_id: String, count: usize) -> Result<(), ServerFnError> {
    if !get_is_streamer(user_id).await.unwrap() {
        Ok(())
    } else {
        let pokemon = crate::app::get_pokemon(cx, "sv".to_string(), count).await?;
        POLL_VOTES.write().unwrap().clear();
        let mut result_lock = POLL_RESULT.write().unwrap();
        *result_lock = None;
        let mut poll_lock = CURRENT_POLL.write().unwrap();
        *poll_lock = (true, pokemon);
        let _ = CURRENT_POLL_CHANNEL.send(&poll_lock).await;
        let _ = POLL_RESULT_CHANNEL.send(&result_lock).await;
        Ok(())
    }
}

#[server(EndPoll, "/api")]
pub async fn end_poll(user_id: String) -> Result<(), ServerFnError> {
    if !get_is_streamer(user_id).await.unwrap() {
        Ok(())
    } else {
        {
            let mut votes_lock = POLL_VOTES.write().unwrap();
            let mut poll_lock = CURRENT_POLL.write().unwrap();
            let mut results_lock = POLL_RESULT.write().unwrap();
            let count = poll_lock.1.len();
            let mut results = vec![0; count];
            votes_lock.iter().for_each(|(_, &vote)| {
                results[vote] += 1;
            });

            let mut index = 0;
            let mut max = 0;
            results.into_iter().enumerate().for_each(|(i, votes)| {
                if votes > max {
                    max = votes;
                    index = i;
                }
            });

            *results_lock = Some(poll_lock.1[index].clone());
            *poll_lock = (false, vec![]);
            votes_lock.clear();
            let _ = CURRENT_POLL_CHANNEL.send(&poll_lock).await;
            let _ = POLL_RESULT_CHANNEL.send(&results_lock).await;
        }
        Ok(())
    }
}

#[component]
pub fn streamer_buttons(cx: Scope, user_id: String) -> impl IntoView {
    let generate_poll = create_server_action::<GeneratePoll>(cx);
    let end_poll = create_server_action::<EndPoll>(cx);
    let clone = user_id.clone();
    view! {
        cx,
        <button class="px-3 py-2 bg-green-500 rounded-md" on:click=move |_| generate_poll.dispatch(GeneratePoll{user_id: user_id.clone(), count: 4})>"Generate"</button>
        <button class="px-3 py-2 bg-red-500 rounded-md" on:click=move |_| end_poll.dispatch(EndPoll{user_id: clone.clone()})>"End Poll"</button>
    }
}
