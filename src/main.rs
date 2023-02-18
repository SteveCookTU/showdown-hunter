use cfg_if::cfg_if;

use showdown_hunter::app;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::collections::HashMap;
        use leptos::*;
        use showdown_hunter::app::*;
        use actix_files::Files;
        use actix_web::*;
        use smog_strat_dex_rs::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};

        #[get("/style.css")]
        async fn css() -> impl Responder {
            actix_files::NamedFile::open_async("./style/output.css").await
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            let mut basics = HashMap::new();

            basics.insert(Generation::ScarletViolet as u8, Client::get_basics(Generation::ScarletViolet).await.unwrap());

            app::register_server_functions();

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).await.unwrap();

            let addr = conf.leptos_options.site_address.clone();

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|cx| view! { cx, <App/> });

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
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
