#[macro_use]
extern crate rocket;
use chrono::NaiveDateTime;
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_dyn_templates::Template;
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

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
    let punch = sqlx::query_as("SELECT * FROM punches WHERE user_id = $1 ORDER BY id DESC LIMIT 1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(punch))
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

// home route
#[get("/home")]
async fn home(state: &State<MyState>) -> Result<Template, BadRequest<String>> {
    match user_list(state).await {
        Ok(users) => {
            let mut context = HashMap::new();
            context.insert("users", users.into_inner());
            Ok(Template::render("home", &context))
        }
        Err(e) => Err(e),
    }
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
        .mount("/", routes![home])
        .attach(Template::fairing())
        .manage(state);

    Ok(rocket.into())
}

// Define the function to initialize your database
async fn initialize_db(pool: &PgPool) -> Result<(), CustomError> {
    let init_sql = include_str!("../init.sql");
    let anytime_sql = include_str!("../anytime.sql");

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

    // Execute anytime.sql
    for command in anytime_sql.split(';') {
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

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "punch", rename_all = "lowercase")]
enum InOut {
    In,
    Out,
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
