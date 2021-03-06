#![feature(crate_in_paths)]
#![feature(nll)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate bytes;
#[macro_use]
extern crate diesel;
extern crate base64;
extern crate dotenv;
extern crate elba;
extern crate env_logger;
// extern crate semver;
extern crate tar;
#[macro_use]
extern crate failure;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

mod index;
mod package;
mod router;
mod schema;
mod user;
mod util;

use std::env;

use actix::prelude::*;
use actix_web::{middleware, server, App};

use crate::index::storage::FileStorage;
use crate::util::{
    database::{self, Database},
    Config,
};

lazy_static! {
    pub static ref CONFIG: Config = Config::from_env();
}

#[derive(Clone)]
pub struct AppState {
    pub db: Addr<Database>,
    pub storage: Addr<FileStorage>,
}

fn main() {
    dotenv::dotenv().ok();
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "actix_web=debug,info,warn");
    env_logger::init();

    let address = env::var("BIND_TO").expect("BIND_TO not set!");
    let sys = System::new("elba-backaned");

    let db = database::connect();
    let db_actor = SyncArbiter::start(num_cpus::get() * 4, move || db.clone());
    let storage = SyncArbiter::start(num_cpus::get(), move || FileStorage);
    let app_state = AppState {
        db: db_actor,
        storage,
    };

    server::new(move || {
        let app = App::with_state(app_state.clone()).middleware(middleware::Logger::default());
        router::router(app)
    }).bind(&address)
        .expect(&format!("Can't bind to {}", &address))
        .start();

    sys.run();
}
