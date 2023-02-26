use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::collections::HashMap;
        use leptos::*;
        use showdown_hunter::app::*;
        use actix_files::Files;
        use actix_web::*;
        use smog_strat_dex_rs::*;
        use futures::StreamExt;
        use showdown_hunter::poll::CURRENT_POLL_CHANNEL;
        use showdown_hunter::poll::POLL_RESULT_CHANNEL;

        use leptos_actix::{generate_route_list, LeptosRoutes};

        #[get("/style.css")]
        async fn css() -> impl Responder {
            actix_files::NamedFile::open_async("./style/output.css").await
        }

        #[get("/api/poll")]
        async fn poll() -> impl Responder {
            let stream = futures::stream::once(async { showdown_hunter::poll::get_current_poll().await.unwrap_or_default() }).chain(CURRENT_POLL_CHANNEL.clone()).map(|value| {
                Ok(web::Bytes::from(format!("event: poll\ndata: {}\n\n", serde_json::to_string(&value).unwrap_or_default()))) as actix_web::Result<web::Bytes>
            });

            HttpResponse::Ok().insert_header(("Content-Type", "text/event-stream")).streaming(stream)
        }

        #[get("/api/result")]
        async fn result() -> impl Responder {
            let stream = futures::stream::once(async { showdown_hunter::poll::get_poll_result().await.unwrap_or_default() }).chain(POLL_RESULT_CHANNEL.clone()).map(|value| {
                Ok(web::Bytes::from(format!("event: result\ndata: {}\n\n", serde_json::to_string(&value).unwrap_or_default()))) as actix_web::Result<web::Bytes>
            });

            HttpResponse::Ok().insert_header(("Content-Type", "text/event-stream")).streaming(stream)
        }

        #[get("/api/image/{name}")]
        async fn image(path: web::Path<String>) -> impl Responder {
            let name = path.into_inner();
            if let Ok(resp) = reqwest::get(format!(
                "https://www.smogon.com/dex/media/sprites/xy/{}.gif",
                name
            )).await {
                if let Ok(bytes) = resp.bytes().await {
                    return HttpResponse::Ok().content_type("image/gif").body(bytes);
                }
            }
            HttpResponse::BadRequest().finish()
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            let mut basics = HashMap::new();
            basics.insert(Generation::ScarletViolet as u8, Client::get_basics(Generation::ScarletViolet).await.unwrap());

            showdown_hunter::register_server_functions();

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).await.unwrap();

            let addr = conf.leptos_options.site_addr.clone();

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|cx| view! { cx, <App/> });

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
                    .service(poll)
                    .service(result)
                    .service(image)
                    .app_data(web::Data::new(basics.clone()))
                    .service(css)
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .leptos_routes(leptos_options.to_owned(), routes.to_owned(), |cx| view! { cx, <App/> })
                    .service(Files::new("/", &site_root))
            })
            .bind(&addr)?
            .run()
            .await
        }
    } else {
        pub fn main() {}
    }
}
