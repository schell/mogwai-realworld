use serde::{Deserialize, Serialize};

/// The conduit API URL
pub const API_URL: &'static str = "https://conduit.productionready.io/api";

pub mod request {
    //! Sending conduit API requests.
    use mogwai::prelude::{utils, JsFuture};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use snafu::{OptionExt, ResultExt, Snafu};
    use std::collections::HashMap;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

    #[derive(Debug, Deserialize)]
    pub struct ApiResponseErrors {
        errors: HashMap<String, Vec<String>>,
    }

    /// An enumeration of all API errors.
    #[derive(Debug, Snafu)]
    pub enum Error {
        #[snafu(display("could not construct request"))]
        ConstructRequest,
        #[snafu(display("could not create request headers"))]
        CantCreateHeaders,
        #[snafu(display("could not set request headers"))]
        CantAppendHeaders,
        #[snafu(display("request failure"))]
        RequestFailure,
        #[snafu(display("response was malformed"))]
        MalformedResponse,
        #[snafu(display("response was not json"))]
        FailedJson,
        #[snafu(display("timed out awaiting json"))]
        FailedAwaitingJson,
        #[snafu(display("could not deserialize response json: {}", source))]
        DeserializeFailure {
            source: serde_json::Error,
        },
        #[snafu(display("could not serialize request json: {}", source))]
        SerializeFailure {
            source: serde_json::Error,
        },
        ResponseErrors {
            errors: Vec<(String, Vec<String>)>,
        },
    }

    async fn send_request<T: DeserializeOwned>(req: Request) -> Result<T, Error> {
        let resp: Response = JsFuture::from(utils::window().fetch_with_request(&req))
            .await
            .ok()
            .with_context(|| RequestFailure)?
            .dyn_into()
            .ok()
            .with_context(|| MalformedResponse)?;
        let js_value: JsValue = JsFuture::from(resp.json().ok().with_context(|| FailedJson)?)
            .await
            .ok()
            .with_context(|| FailedAwaitingJson)?;
        js_value
            .into_serde::<T>()
            .with_context(|| DeserializeFailure)
            .map_err(|e| match js_value.into_serde::<ApiResponseErrors>() {
                Ok(ApiResponseErrors { errors }) => Error::ResponseErrors {
                    errors: errors.into_iter().collect(),
                },
                _ => e,
            })
    }

    fn req_init<T: Serialize>(o_body: Option<&T>) -> Result<RequestInit, Error> {
        let mut opts = RequestInit::new();
        let headers = Headers::new().ok().with_context(|| CantCreateHeaders)?;
        headers
            .append("Content-Type", "application/json; charset=utf-8")
            .ok()
            .with_context(|| CantAppendHeaders)?;
        opts.headers(&headers);
        opts.mode(RequestMode::Cors);
        if let Some(body) = o_body {
            let json_body = serde_json::to_string(body).with_context(|| SerializeFailure)?;
            opts.body(Some(&JsValue::from(json_body)));
        }
        Ok(opts)
    }

    pub async fn api<T: Serialize, S: DeserializeOwned>(
        url: &str,
        method: &str,
        o_body: Option<&T>,
    ) -> Result<S, Error> {
        let mut opts = req_init(o_body)?;
        opts.method(method);
        opts.mode(RequestMode::Cors);

        let req = Request::new_with_str_and_init(url, &opts)
            .ok()
            .with_context(|| ConstructRequest)?;

        send_request::<S>(req).await
    }
}

/// A user.
///
/// {"user":{"id":118083,"email":"schell@zyghost.com","createdAt":"2020-10-06T22:42:31.921Z","updatedAt":"2020-10-06T22:42:31.928Z","username":"schell","bio":null,"image":null,"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MTE4MDgzLCJ1c2VybmFtZSI6InNjaGVsbCIsImV4cCI6MTYwNzIwODE1MX0.EgesXVrJkTTd53KCzqldEBWV1X_-PZKcA0zcn9ZlG7U"}}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct UserWrapper {
    user: User,
}

/// POST /api/users/login
pub async fn auth_user(user: UserRegistration) -> Result<User, request::Error> {
    let url = format!("{}/users/login", API_URL);
    let UserWrapper { user } =
        request::api(&url, "POST", Some(&UserRegistrationWrapper { user })).await?;
    Ok(user)
}

#[derive(Clone, Debug, Serialize)]
pub struct UserRegistration {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
struct UserRegistrationWrapper {
    user: UserRegistration,
}

/// POST /api/users
pub async fn register_user(user: UserRegistration) -> Result<User, request::Error> {
    let url = format!("{}/users", API_URL);
    let UserWrapper { user } =
        request::api(&url, "POST", Some(&UserRegistrationWrapper { user })).await?;
    Ok(user)
}
