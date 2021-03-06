use warp::{Filter, Rejection, Reply};

use crate::handlers::{admin, user};
use crate::model::enrollee::Status;
use crate::model::user::Role;
use crate::Application;
use chrono::NaiveDate;

mod jwt;

pub fn routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name))
        .or(user_routes(app))
        .or(admin_routes(app))
}

fn user_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("user").and(auth_routes(app))
}

fn auth_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(user::auth::register);
    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(user::auth::login);
    let logout = warp::path("logout")
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![]))
        .and(jwt::refresh_filter())
        .and_then(user::auth::logout);
    let refresh_session = warp::path("refresh-session")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and(jwt::refresh_filter())
        .and_then(user::auth::refresh_session);
    let routes = register.or(login).or(logout).or(refresh_session);
    warp::path("auth").and(routes)
}

fn admin_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("admin").and(queue_routes(app))
}

fn queue_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let dates = warp::path("dates")
        .and(warp::get())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::dates);
    let enrollees = warp::path("enrollees")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::enrollees);
    let processed = warp::path!("status" / i64 / Status)
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::status);
    let update = warp::path("update")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::update);
    let students_queue = warp::path("students-queue")
        .and(warp::get())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::students_queue);
    let relevant_time = warp::path!("relevant-time" / NaiveDate)
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::relevant_time);
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::queue::register);
    let routes = dates
        .or(enrollees)
        .or(processed)
        .or(update)
        .or(students_queue)
        .or(relevant_time)
        .or(register);
    warp::path("queue").and(routes)
}

pub fn with_app(
    app: &'static Application,
) -> impl Filter<Extract = (&Application,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app)
}
