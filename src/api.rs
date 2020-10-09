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

    impl From<Error> for Vec<String> {
        fn from(err: Error) -> Vec<String> {
            match err {
                Error::ResponseErrors { errors } => errors
                    .into_iter()
                    .flat_map(|(name, descs)| -> Vec<String> {
                        descs
                            .into_iter()
                            .map(|desc| format!("{} {}", name, desc))
                            .collect()
                    })
                    .collect(),
                _ => vec![format!("{}", err)],
            }
        }
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

    fn req_init<T: Serialize>(
        o_body: Option<&T>,
        o_auth: Option<&str>,
    ) -> Result<RequestInit, Error> {
        let mut opts = RequestInit::new();
        let headers = Headers::new().ok().with_context(|| CantCreateHeaders)?;
        headers
            .append("Content-Type", "application/json; charset=utf-8")
            .ok()
            .with_context(|| CantAppendHeaders)?;
        if let Some(token) = o_auth {
            headers
                .append("Authorization", &format!("Token {}", token))
                .ok()
                .with_context(|| CantAppendHeaders)?;
        }
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
        o_auth: Option<&str>,
    ) -> Result<S, Error> {
        let mut opts = req_init(o_body, o_auth)?;
        opts.method(method);
        opts.mode(RequestMode::Cors);

        let req = Request::new_with_str_and_init(url, &opts)
            .ok()
            .with_context(|| ConstructRequest)?;

        send_request::<S>(req).await
    }
}

/// A user.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UserWrapper<T> {
    user: T,
}

/// A user's registration or login.
#[derive(Clone, Debug, Serialize)]
pub struct UserRegistration {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// POST /api/users/login
pub async fn auth_user(user: UserRegistration) -> Result<User, request::Error> {
    let url = format!("{}/users/login", API_URL);
    let UserWrapper { user } =
        request::api(&url, "POST", Some(&UserWrapper { user }), None).await?;
    Ok(user)
}

/// POST /api/users
pub async fn register_user(user: UserRegistration) -> Result<User, request::Error> {
    let url = format!("{}/users", API_URL);
    let UserWrapper { user } =
        request::api(&url, "POST", Some(&UserWrapper { user }), None).await?;
    Ok(user)
}

#[derive(Clone, Debug, Serialize)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub password: Option<String>,
}

impl Default for UserUpdate {
    fn default() -> Self {
        UserUpdate {
            email: None,
            username: None,
            bio: None,
            image: None,
            password: None,
        }
    }
}

/// PUT /api/user
pub async fn update_user(user: UserUpdate, token: &str) -> Result<User, request::Error> {
    let url = format!("{}/user", API_URL);
    let UserWrapper { user } =
        request::api(&url, "PUT", Some(&UserWrapper { user }), Some(token)).await?;
    Ok(user)
}
