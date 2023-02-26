use leptos::html::P;
use leptos::*;

pub fn format_set(cx: Scope, set: String) -> Vec<HtmlElement<P>> {
    set.split('\n')
        .map(|s| {
            view! {
                cx,
                <p>{s.to_string()}</p>
            }
        })
        .collect::<Vec<_>>()
}

#[component]
pub fn pokemon(cx: Scope, name: String, set: String) -> impl IntoView {
    view! {
        cx,
        <div class="flex flex-col place-self-center">
            <div class="place-self-center">
                <div class="w-16 h-16" style={format!("background-repeat: no-repeat; background-size: contain; background-position: center center; background-image: url(\"/api/image/{}\")", name.to_lowercase().replace(' ', "-"))}></div>
            </div>
            <div>
                {format_set(cx, set)}
            </div>
        </div>

    }
}
