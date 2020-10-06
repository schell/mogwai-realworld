use mogwai::prelude::{utils, JsCast, JsFuture, JsValue};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use web_sys::{Request, RequestInit, RequestMode, Response};

pub const API_URL: &'static str = "https://conduit.productionready.io/api";

/// An enumeration of all API errors.
#[derive(Debug, Snafu)]
pub enum Error {
    ConstructRequest { js: JsValue },
    RequestFailure { js: JsValue },
    MalformedResponse { js: JsValue },
    FailedJson { js: JsValue },
    FailedAwaitingJson { js: JsValue },
    DeserializeFailure { source: serde_json::Error },
}

async fn send_request<T: DeserializeOwned>(req: Request) -> Result<T, Error> {
    let resp: Response = JsFuture::from(utils::window().fetch_with_request(&req))
        .await
        .map_err(|js| Error::RequestFailure { js })?
        .dyn_into()
        .map_err(|js| Error::MalformedResponse { js })?;
    let js_value: JsValue = JsFuture::from(resp.json().map_err(|js| Error::FailedJson { js })?)
        .await
        .map_err(|js| Error::FailedAwaitingJson { js })?;
    js_value.into_serde().with_context(|| DeserializeFailure)
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    email: String,
    token: (),
    username: String,
    bio: String,
    // TODO: Can this be an http::url::Url?
    image: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct UserWrapper {
    user: User,
}

/// POST /api/users/login
pub async fn auth_user() -> Result<User, Error> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let req = Request::new_with_str_and_init(
        &format!("{}/users/login", API_URL),
        &opts,
    )
        .map_err(|js| Error::ConstructRequest{ js })?;

    let user = send_request::<User>(req).await?;
    Ok(user)
}
