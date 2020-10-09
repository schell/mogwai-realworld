use log::trace;
use mogwai::prelude::*;

use crate::{
    components::{login::Login, register::Register, settings::Settings},
    page,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    Login,
    Register,
    Settings,
    Editor {
        o_slug: Option<String>,
    },
    Article {
        slug: String,
    },
    Profile {
        username: String,
        is_favorites: bool,
    },
}

impl TryFrom<&str> for Route {
    type Error = String;

    fn try_from(s: &str) -> Result<Route, String> {
        trace!("route try_from: {}", s);
        // remove the scheme, if it has one
        let hash_split = s.split("#").collect::<Vec<_>>();
        let after_hash = match hash_split.as_slice() {
            [_, after] => Ok(after),
            _ => Err(format!("route must have a hash: {}", s)),
        }?;

        let paths: Vec<&str> = after_hash.split("/").collect::<Vec<_>>();
        trace!("route paths: {:?}", paths);

        match paths.as_slice() {
            [""] => Ok(Route::Home),
            ["", ""] => Ok(Route::Home),
            ["", "login"] => Ok(Route::Login),
            ["", "register"] => Ok(Route::Register),
            ["", "settings"] => Ok(Route::Settings),
            ["", "editor"] => Ok(Route::Editor { o_slug: None }),
            ["", "editor", slug] => Ok(Route::Editor {
                o_slug: Some(slug.to_string()),
            }),
            ["", "article", slug] => Ok(Route::Article {
                slug: slug.to_string(),
            }),
            ["", "profile", username] => Ok(Route::Profile {
                username: username.to_string(),
                is_favorites: false,
            }),
            ["", "profile", username, "favorites"] => Ok(Route::Profile {
                username: username.to_string(),
                is_favorites: true,
            }),
            r => Err(format!("unsupported route: {:?}", r)),
        }
    }
}

impl TryFrom<String> for Route {
    type Error = String;

    fn try_from(s: String) -> Result<Route, String> {
        Route::try_from(s.as_str())
    }
}

impl From<&Route> for ViewBuilder<HtmlElement> {
    fn from(route: &Route) -> Self {
        match route {
            Route::Home => page::home().into(),
            Route::Login => Gizmo::from(Login::default()).view_builder(),
            Route::Register => {
                let register = Gizmo::from(Register::default());
                register.view_builder()
            }
            Route::Settings => Gizmo::from(Settings::default()).view_builder(),
            Route::Editor { o_slug } => page::editor(o_slug).into(),
            Route::Article { slug } => page::article(slug).into(),
            Route::Profile {
                username,
                is_favorites,
            } => page::profile(username, *is_favorites).into(),
        }
    }
}

impl From<&Route> for View<HtmlElement> {
    fn from(route: &Route) -> Self {
        ViewBuilder::from(route).into()
    }
}

impl Route {
    pub fn nav_home_class(&self) -> String {
        match self {
            Route::Home => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_editor_class(&self) -> String {
        match self {
            Route::Editor { .. } => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_settings_class(&self) -> String {
        match self {
            Route::Settings { .. } => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_register_class(&self) -> String {
        match self {
            Route::Register => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_login_class(&self) -> String {
        match self {
            Route::Login => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn nav_profile_class(&self) -> String {
        match self {
            Route::Profile{ .. } => "nav-link active",
            _ => "nav-link",
        }
        .to_string()
    }

    pub fn as_title(&self) -> String {
        match self {
            Route::Home => "Home".into(),
            Route::Register => "Sign Up".into(),
            Route::Login => "Sign In".into(),
            Route::Editor { .. } => "Editor".into(),
            Route::Settings => "Settings".into(),
            Route::Article { .. } => "Article".into(),
            Route::Profile { .. } => "Profile".into(),
        }
    }

    pub fn as_hash(&self) -> String {
        match self {
            Route::Home => "/".into(),
            Route::Register => "/register".into(),
            Route::Login => "/login".into(),
            Route::Editor { o_slug } => {
                if let Some(slug) = o_slug {
                    format!("/editor/{}", slug)
                } else {
                    "/editor".into()
                }
            }
            Route::Settings => "/settings".into(),
            Route::Article { slug } => format!("/article/{}", slug),
            Route::Profile {
                username,
                is_favorites,
            } => {
                if *is_favorites {
                    format!("/profile/{}/favorites", username)
                } else {
                    format!("/profile/{}", username)
                }
            }
        }
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[test]
    fn can_convert_string_to_route() {
        let s = "https://localhost:8080/#/";
        assert_eq!(Route::try_from(s), Ok(Route::Home));
    }
}
