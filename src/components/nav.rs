//! The navigation component.
//!
//! Receives model messages about changing routes and outputs view messages about changing
//! routes.
#![allow(unused_braces)]
use mogwai::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HashChangeEvent;

use super::super::route::Route;

pub struct Nav {
    pub current_route: Route,
}

impl Default for Nav {
    fn default() -> Nav {
        Nav {
            current_route: Route::try_from(utils::window().location().href().unwrap_throw())
                .unwrap_or_else(|_| Route::Home),
        }
    }
}

#[derive(Clone)]
pub enum NavModel {
    HashChange { route: Route },
}

#[derive(Clone)]
pub struct NavView {
    pub route: Route,
}

impl Component for Nav {
    type ModelMsg = NavModel;
    type ViewMsg = NavView;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &NavModel, tx: &Transmitter<NavView>, _sub: &Subscriber<NavModel>) {
        match msg {
            NavModel::HashChange { route } => tx.send(&NavView {
                route: route.clone(),
            }),
        }
    }

    fn view(&self, tx: &Transmitter<NavModel>, rx: &Receiver<NavView>) -> ViewBuilder<HtmlElement> {
        let home_class = (
            self.current_route.nav_home_class(),
            rx.branch_map(|msg| msg.route.nav_home_class()),
        );
        let editor_class = (
            self.current_route.nav_editor_class(),
            rx.branch_map(|msg| msg.route.nav_editor_class()),
        );
        let settings_class = (
            self.current_route.nav_settings_class(),
            rx.branch_map(|msg| msg.route.nav_settings_class()),
        );
        let register_class = (
            self.current_route.nav_register_class(),
            rx.branch_map(|msg| msg.route.nav_register_class()),
        );

        builder! {
            <nav
                class="navbar navbar-light"
                window:hashchange=tx.contra_filter_map(|ev:&Event| {
                    let hev = ev.dyn_ref::<HashChangeEvent>().unwrap().clone();
                    let route = Route::try_from(hev.new_url().as_str()).ok()?;
                    Some(NavModel::HashChange{ route })
                })>
                <div class="container">
                    <a class="navbar-brand" href="#">"conduit"</a>
                    <ul class="nav navbar-nav pull-xs-right">
                        <li class="nav-item">
                            <a class=home_class href="#/">" Home"</a>
                        </li>
                        <li class="nav-item">
                            <a class=editor_class href="#/editor">
                                <i class="ion-compose"></i>
                                " New Post"
                            </a>
                        </li>
                        <li class="nav-item">
                            <a class=settings_class href="#/settings">
                                <i class="ion-gear-a"></i>
                                " Settings"
                            </a>
                        </li>
                        <li class="nav-item">
                            <a class=register_class href="#/register">" Sign up"</a>
                        </li>
                    </ul>
                </div>
            </nav>
        }
    }
}
