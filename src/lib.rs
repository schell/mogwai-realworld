#![allow(unused_braces)]
use log::{trace, Level};
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::{prelude::*, UnwrapThrowExt};
use web_sys::HashChangeEvent;

mod api;
mod page;
mod route;
use route::*;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


struct App {
    current_route: Route,
}


impl Default for App {
    fn default() -> App {
        App {
            current_route: Route::try_from(utils::window().location().href().unwrap_throw())
                .unwrap_or_else(|_| Route::Home),
        }
    }
}


#[derive(Clone)]
enum AppModel {
    HashChange { route: Route },
}


#[derive(Clone)]
enum AppView {
    NewPage { page: View<HtmlElement> },
}


impl Component for App {
    type ModelMsg = AppModel;
    type ViewMsg = AppView;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &AppModel, tx: &Transmitter<AppView>, _sub: &Subscriber<AppModel>) {
        match msg {
            AppModel::HashChange { route } => {
                let page = View::from(route);
                tx.send(&AppView::NewPage { page })
            }
        }
    }

    fn view(&self, tx: &Transmitter<AppModel>, rx: &Receiver<AppView>) -> ViewBuilder<HtmlElement> {
        builder! (
            <slot patch:children=rx.branch_filter_map(|msg| match msg {
                AppView::NewPage{ page } => Some(Patch::Replace{ index: 1, value: page.clone() })
            })>
                <nav
                    class="navbar navbar-light"
                    window:hashchange=tx.contra_filter_map(|ev:&Event| {
                        let hev = ev.dyn_ref::<HashChangeEvent>().unwrap().clone();
                        let route = Route::try_from(hev.new_url().as_str()).ok()?;
                        Some(AppModel::HashChange{ route })
                    })>
                    <div class="container">
                        <a class="navbar-brand" href="#">"conduit"</a>
                        <ul class="nav navbar-nav pull-xs-right">
                            <li class="nav-item">
                                // TODO: Add "active" class when you're on that page"
                                <a class="nav-link active" href="#/">" Home"</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#/editor">
                                    <i class="ion-compose"></i>
                                    " New Post"
                                </a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#/settings">
                                    <i class="ion-gear-a"></i>
                                    " Settings"
                                </a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#/register">" Sign up"</a>
                            </li>
                        </ul>
                    </div>
                </nav>

                // This node gets replaced every time we send a patch from the parent node ^
                {ViewBuilder::from(&self.current_route)}

                <footer>
                    <div class="container">
                        <a href="/" class="logo-font">"conduit"</a>
                        <span class="attribution">
                            "An interactive learning project from "
                            <a href="https://thinkster.io">"Thinkster"</a>". "
                            "Code & design licensed under MIT."
                        </span>
                    </div>
                </footer>
            </slot>
        )
    }
}


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    Gizmo::from(App::default()).run()
}
