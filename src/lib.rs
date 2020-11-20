#![allow(unused_braces)]
use log::Level;
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;

mod api;
mod components;
mod page;
mod route;
mod store;
mod widgets;

use components::{
    login::Login,
    nav::Nav,
    register::Register,
};
use route::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub enum Page {
    Login(Gizmo<Login>),
    Register(Gizmo<Register>),
}

struct App {
    nav: Gizmo<Nav>,
}

impl Default for App {
    fn default() -> App {
        let nav = Gizmo::from(Nav::default());
        App { nav }
    }
}

#[derive(Clone)]
enum AppModel {
    HashChange { route: Route },
}

#[derive(Clone)]
enum AppView {
    NewPage {
        page: View<HtmlElement>,
        route: Route,
    },
}

impl Component for App {
    type ModelMsg = AppModel;
    type ViewMsg = AppView;
    type DomNode = HtmlElement;

    fn bind(&self, sub: &Subscriber<AppModel>) {
        // bind the nav's output view messages to our input model messages
        sub.subscribe_filter_map(&self.nav.recv, |msg| msg.route().map(|r| AppModel::HashChange {
            route: r.clone()
        }));
    }

    fn update(&mut self, msg: &AppModel, tx: &Transmitter<AppView>, _sub: &Subscriber<AppModel>) {
        match msg {
            AppModel::HashChange { route } => {
                let page = View::from(route);
                tx.send(&AppView::NewPage {
                    page,
                    route: route.clone(),
                })
            }
        }
    }

    fn view(&self, _: &Transmitter<AppModel>, rx: &Receiver<AppView>) -> ViewBuilder<HtmlElement> {
        builder! {
            <slot patch:children=rx.branch_filter_map(|msg| match msg {
                AppView::NewPage{ page, .. } => Some(Patch::Replace{ index: 1, value: page.clone() }),
            })>
                {self.nav.view_builder()}

                // This node gets replaced every time we send a patch from the parent node ^
                {ViewBuilder::from(&self.nav.state_ref().current_route)}

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
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    View::from(Gizmo::from(App::default())).run()
}
