use leptos::*;
use leptos_meta::*;
use leptos_router::*;

pub mod components;
pub mod pages;
pub mod api;

use pages::{Home, Settings};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/openzax-ui.css"/>
        <Title text="OpenZax - Secure AI Development Assistant"/>
        <Meta name="description" content="OpenZax is a secure AI development assistant built with Rust"/>
        
        <Router>
            <main class="app-container">
                <Routes>
                    <Route path="/" view=Home/>
                    <Route path="/settings" view=Settings/>
                </Routes>
            </main>
        </Router>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    
    tracing_wasm::set_as_global_default();
    
    leptos::mount_to_body(App);
}
