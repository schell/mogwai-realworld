//! The navigation component.
//!
//! Receives model messages about changing routes and outputs view messages about changing
//! routes.
#![allow(unused_braces)]
use mogwai::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HashChangeEvent;

use crate::{api::User, route::Route, store};

pub struct Nav {
    pub current_route: Route,
    pub o_user: Option<User>,
}

impl Default for Nav {
    fn default() -> Nav {
        let current_route = Route::try_from(utils::window().location().href().unwrap_throw())
            .unwrap_or_else(|_| Route::Home);
        let o_user = store::read_user().ok();
        Nav {
            current_route,
            o_user,
        }
    }
}

fn signed_out_view_builder(
    home_class: Effect<String>,
    login_class: Effect<String>,
    register_class: Effect<String>,
) -> ViewBuilder<HtmlElement> {
    builder! {
        <ul class="nav navbar-nav pull-xs-right">
            <li class="nav-item">
                <a class=home_class href="#/">" Home"</a>
            </li>
            <li class="nav-item">
            <a class=login_class href="#/login">" Sign in"</a>
            </li>
            <li class="nav-item">
                <a class=register_class href="#/register">" Sign up"</a>
            </li>
        </ul>
    }
}

fn signed_in_view_builder(
    user: &User,
    home_class: Effect<String>,
    editor_class: Effect<String>,
    settings_class: Effect<String>,
    profile_class: Effect<String>,
) -> ViewBuilder<HtmlElement> {
    builder! {
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
                <a class=profile_class href=format!("#/profile/{}", user.username)>
                    {format!(" {}", user.username)}
                </a>
            </li>
        </ul>

    }
}

fn list_view(
    route: &Route,
    o_user: Option<&User>,
    rx: &Receiver<NavView>,
) -> ViewBuilder<HtmlElement> {
    let home_class: Effect<String> = (
        route.nav_home_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_home_class())),
    )
        .into();
    let editor_class: Effect<String> = (
        route.nav_editor_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_editor_class())),
    )
        .into();
    let settings_class: Effect<String> = (
        route.nav_settings_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_settings_class())),
    )
        .into();
    let register_class: Effect<String> = (
        route.nav_register_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_register_class())),
    )
        .into();
    let login_class: Effect<String> = (
        route.nav_register_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_register_class())),
    )
        .into();
    let profile_class: Effect<String> = (
        route.nav_register_class(),
        rx.branch_filter_map(|msg| msg.route().map(|r| r.nav_profile_class())),
    )
        .into();

    if let Some(user) = o_user.as_ref() {
        signed_in_view_builder(
            user,
            home_class.clone(),
            editor_class.clone(),
            settings_class.clone(),
            profile_class,
        )
    } else {
        signed_out_view_builder(home_class, login_class, register_class)
    }
}

#[derive(Clone)]
pub enum NavModel {
    HashChange(Route),
}

#[derive(Clone)]
pub enum NavView {
    Route(Route),
    PatchListView(Patch<View<HtmlElement>>),
}

impl NavView {
    pub fn route(&self) -> Option<Route> {
        match self {
            NavView::Route(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub fn patch_list_view(&self) -> Option<Patch<View<HtmlElement>>> {
        match self {
            NavView::PatchListView(patch) => Some(patch.clone()),
            _ => None,
        }
    }
}

impl Component for Nav {
    type ModelMsg = NavModel;
    type ViewMsg = NavView;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &NavModel, tx: &Transmitter<NavView>, _sub: &Subscriber<NavModel>) {
        match msg {
            NavModel::HashChange(route) => {
                if route != &self.current_route {
                    self.current_route = route.clone();
                    mogwai::utils::document().set_title(&route.as_title());
                    tx.send(&NavView::Route(route.clone()));

                    let o_user = store::read_user().ok();
                    if o_user != self.o_user {
                        self.o_user = o_user;
                        tx.send(&NavView::PatchListView(Patch::Replace {
                            index: 1,
                            value: View::from(list_view(
                                &self.current_route,
                                self.o_user.as_ref(),
                                &tx.spawn_recv(),
                            )),
                        }));
                    }
                }
            }
        }
    }

    fn view(&self, tx: &Transmitter<NavModel>, rx: &Receiver<NavView>) -> ViewBuilder<HtmlElement> {
        let route = self.current_route.clone();

        builder! {
            <nav
                class="navbar navbar-light"
                post:build=tx.contra_map(move |_| NavModel::HashChange(route.clone()))
                window:hashchange=tx.contra_filter_map(|ev:&Event| {
                    let hev = ev.dyn_ref::<HashChangeEvent>().unwrap().clone();
                    let route = Route::try_from(hev.new_url().as_str()).ok()?;
                    Some(NavModel::HashChange( route ))
                })>
                <div
                    class="container"
                    patch:children=rx.branch_filter_map(|msg| msg.patch_list_view())>
                    <a class="navbar-brand" href="#">"conduit"</a>
                    {list_view(&self.current_route, self.o_user.as_ref(), rx)}
                </div>
            </nav>
        }
    }
}
