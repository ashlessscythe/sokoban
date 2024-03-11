#[macro_use]
extern crate rocket;
use chrono::NaiveDateTime;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::BadRequest;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(FromForm)]
struct LoginForm {
    user_token: String,
}

#[post("/", data = "<login_form>")]
fn login(cookies: &CookieJar<'_>, login_form: Form<LoginForm>) -> Redirect {
    // In a real-world scenario, validate the user_token against your data store
    if is_valid_token(&login_form.user_token) {
        cookies.add_private(Cookie::new("user_token", login_form.user_token.clone()));
        Redirect::to(uri!("/userlist"))
    } else {
        Redirect::to(uri!("/login"))
    }
}

#[get("/login_form")]
fn login_form() -> Template {
    Template::render("login", context! {})
}

struct Authenticated;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = (); // Using unit type for simplicity; customize as needed.

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // example condition
        if let Some(_cookie) = request.cookies().get_private("authenticated") {
            rocket::outcome::Outcome::Success(Authenticated)
        } else {
            rocket::outcome::Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

// Placeholder for token validation logic
fn is_valid_token(token: &str) -> bool {
    token == "expected_token"
}

#[get("/userlist")]
async fn userlist(_auth: Authenticated, state: &State<MyState>) -> Result<Template, Status> {
    match user_list(state).await {
        Ok(users) => {
            let mut context = HashMap::new();
            context.insert("users", users.into_inner());
            Ok(Template::render("userlist", &context))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

// list of all users
#[get("/users")]
async fn user_list(state: &State<MyState>) -> Result<Json<Vec<User>>, BadRequest<String>> {
    let list = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(list))
}

// list all punches
#[get("/punches")]
async fn punches_list(
    state: &State<MyState>,
) -> Result<Json<Vec<PunchWithUser>>, BadRequest<String>> {
    let list = sqlx::query_as("SELECT * FROM punches_with_user")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(list))
}

// get one user
#[get("/<id>")]
async fn retrieve(id: String, state: &State<MyState>) -> Result<Json<User>, BadRequest<String>> {
    let user = sqlx::query_as("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(user))
}

// create a new user
#[post("/", data = "<data>")]
async fn add(data: Json<User>, state: &State<MyState>) -> Result<Json<User>, BadRequest<String>> {
    // generate if user_id not provided
    let user_id = data
        .user_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let user =
        sqlx::query_as("INSERT INTO users (name, email, user_id) VALUES ($1, $2, $3) RETURNING *")
            .bind(&data.name)
            .bind(&data.email)
            .bind(user_id.to_string())
            .fetch_one(&state.pool)
            .await
            .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(user))
}

// punch user in or out
#[post("/<id>", data = "<data>")]
async fn punch(
    id: String,
    data: Json<PunchRecord>,
    state: &State<MyState>,
) -> Result<Json<PunchRecord>, BadRequest<String>> {
    // insert punch into db
    let punch = sqlx::query_as("INSERT INTO punches (user_id, in_out) VALUES ($1, $2) RETURNING *")
        .bind(id)
        .bind(&data.in_out)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(punch))
}

// get users last punch
#[get("/<id>/last_punch")]
async fn last_punch(
    id: String,
    state: &State<MyState>,
) -> Result<Json<PunchRecord>, BadRequest<String>> {
    let punch = sqlx::query_as::<_, PunchRecord>(
        "SELECT * FROM punches WHERE user_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match punch {
        Ok(Some(punch)) => Ok(Json(punch)),
        Ok(None) => Ok(Json(PunchRecord {
            in_out: InOut::None,
            punch_time: None,
        })),
        Err(e) => Err(BadRequest(e.to_string())),
    }
}

// get all punch records for a user
#[get("/<id>")]
async fn get_user_punches(
    id: String,
    state: &State<MyState>,
) -> Result<Json<Vec<PunchRecord>>, BadRequest<String>> {
    let punch = sqlx::query_as("SELECT * FROM punches WHERE user_id = $1")
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(punch))
}

struct MyState {
    pool: PgPool,
}

// redirect to home
#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!("/home"))
}

// home route
#[get("/home")]
fn home() -> Result<Template, BadRequest<String>> {
    let mut context = HashMap::new();
    context.insert("title", "Home");
    Ok(Template::render("home", &context))
}

// register route
#[get("/register")]
fn register() -> Result<Template, BadRequest<String>> {
    let mut context = HashMap::new();
    context.insert("title", "Register");
    Ok(Template::render("register", &context))
}

// routes
#[shuttle_runtime::main]
async fn rocket(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    if let Err(e) = initialize_db(&pool).await {
        eprintln!("Database initialization failed: {:?}", e);
    }

    let state = MyState { pool };
    let rocket = rocket::build()
        .mount("/user", routes![retrieve, add])
        .mount("/list", routes![user_list, punches_list])
        .mount("/punch", routes![punch, last_punch, get_user_punches])
        .mount("/login", routes![login])
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index, home, login_form, userlist, register])
        .attach(Template::fairing())
        .manage(state);

    Ok(rocket.into())
}

// Define the function to initialize your database
async fn initialize_db(pool: &PgPool) -> Result<(), CustomError> {
    let init_sql = include_str!("../init.sql");

    // Start a transaction
    let mut transaction = pool.begin().await.map_err(CustomError::new)?;

    // Execute init.sql
    for command in init_sql.split(';') {
        let command = command.trim();
        if !command.is_empty() {
            transaction
                .execute(command)
                .await
                .map_err(CustomError::new)?;
        }
    }

    // Commit the transaction
    transaction.commit().await.map_err(CustomError::new)?;

    Ok(())
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "punch", rename_all = "lowercase")]
enum InOut {
    In,
    Out,
    None,
}

#[derive(Deserialize, Serialize, FromRow)]
struct PunchWithUser {
    user_name: String,
    in_out: InOut,
    punch_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, FromRow)]
struct PunchRecord {
    in_out: InOut,
    punch_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, FromRow)]
struct User {
    name: String,
    email: String,
    user_id: Option<String>,
}
