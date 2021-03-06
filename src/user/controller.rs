use std::time::SystemTime;

use actix_web::{
    client, http::StatusCode, AsyncResponder, HttpMessage, HttpResponse, Query, Responder, State,
};
use base64;
use failure::Error;
use futures::prelude::*;

use super::model::{CreateUser, User};
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginReq {
    gh_name: String,
    gh_access_token: String,
}

#[derive(Serialize)]
pub struct LoginRes {
    token: String,
}

#[derive(Deserialize)]
struct GithubRes {
    id: i32,
    email: Option<String>,
    avatar_url: Option<String>,
}

pub fn login((req, state): (Query<LoginReq>, State<AppState>)) -> impl Responder {
    let auth = base64::encode(&format!("{}:{}", req.gh_name, req.gh_access_token));

    client::get("https://api.github.com/user")
        .header("Authorization", format!("Basic {}", auth).as_str())
        .finish()
        .unwrap()
        .send()
        .from_err::<Error>()
        .and_then(|res| {
            if res.status() == StatusCode::OK {
                Ok(res)
            } else {
                Err(format_err!("Bad username or access token to Github."))
            }
        })
        .and_then(|res| res.json().from_err())
        .and_then(move |json: GithubRes| {
            state
                .db
                .send(CreateUser {
                    email: json.email,
                    gh_id: json.id,
                    gh_name: req.gh_name.clone(),
                    gh_access_token: req.gh_access_token.clone(),
                    gh_avatar: json.avatar_url,
                    last_used_at: SystemTime::now(),
                })
                .from_err()
        })
        .flatten()
        .map(|user: User| HttpResponse::Ok().json(LoginRes { token: user.token }))
        .responder()
}
