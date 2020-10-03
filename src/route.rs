use mogwai::prelude::*;
use log::trace;

use super::page as page;


#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    LoginOrRegister,
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
            ["", "login"] => Ok(Route::LoginOrRegister),
            ["", "register"] => Ok(Route::LoginOrRegister),
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
            Route::LoginOrRegister => page::login_register().into(),
            Route::Settings => page::settings().into(),
            Route::Editor{ o_slug: _ } => page::editor().into(),
            Route::Article{ slug } => page::article().into(),
            _ => todo!(),
        }
    }
}


impl From<&Route> for View<HtmlElement> {
    fn from(route: &Route) -> Self {
        ViewBuilder::from(route).into()
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
