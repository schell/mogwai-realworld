use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// {
///     "profile": {
///         "username": "jake",
///         "bio": "I work at statefarm",
///         "image": "https://static.productionready.io/images/smiley-cyrus.jpg",
///         "following": false
///     }
/// }
#[derive(Clone, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub bio: String,
    pub image: String,
    pub following: bool,
}

impl From<User> for UserProfile {
    fn from(user: User) -> UserProfile {
        UserProfile {
            username: user.username,
            bio: user.bio.unwrap_or_else(|| String::new()),
            image: user.image.unwrap_or_else(|| String::new()),
            following: false,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct ProfileWrapper<T> {
    profile: T,
}

/// GET /api/profiles/:username
pub async fn get_profile(
    username: &str,
    o_token: Option<&str>,
) -> Result<UserProfile, request::Error> {
    let url = format!("{}/profiles/{}", API_URL, username);
    let ProfileWrapper { profile } = request::api::<(), _>(&url, "GET", None, o_token).await?;
    Ok(profile)
}

/// {
///     "article": {
///         "slug": "how-to-train-your-dragon",
///         "title": "How to train your dragon",
///         "description": "Ever wonder how?",
///         "body": "It takes a Jacobian",
///         "tagList": ["dragons", "training"],
///         "createdAt": "2016-02-18T03:22:56.637Z",
///         "updatedAt": "2016-02-18T03:48:35.824Z",
///         "favorited": false,
///         "favoritesCount": 0,
///         "author": {
///             "username": "jake",
///             "bio": "I work at statefarm",
///             "image": "https://i.stack.imgur.com/xHWG8.jpg",
///             "following": false
///         }
///     }
/// }
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    //pub created_at: DateTime<Utc>,
    //pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: u32,
    pub author: UserProfile,
}

#[cfg(test)]
mod test_parse_date {
    use chrono::{offset::FixedOffset, DateTime, Utc};

    #[test]
    fn parse_date_time() {
        let s = "2016-02-18T03:48:35.824Z";
        let dt: DateTime<FixedOffset> = todo!();
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Articles {
    pub articles: Vec<Article>,
    pub article_count: u32,
}

/// GET /api/articles
pub async fn get_articles(o_author: Option<&str>, o_tag: Option<&str>, o_favorited: Option<&str>, o_limit: Option<u32>, o_offset: Option<u32>, o_token: Option<&str>) -> Result<Articles, request::Error> {
    let params: Vec<String> = vec![
        o_tag.map(|t| format!("tag={}", t)),
        o_author.map(|a| format!("author={}", a)),
        o_favorited.map(|f| format!("favorited={}", f)),
        o_limit.map(|l| format!("limit={}", l)),
        o_offset.map(|o| format!("offset={}", o)),
    ]
    .into_iter()
    .filter_map(|x| x)
    .collect();
    let params = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    let url = format!("{}/articles{}", API_URL, params);
    request::api::<(), Articles>(&url, "GET", None, o_token).await
}
