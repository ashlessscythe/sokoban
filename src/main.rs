#[macro_use]
extern crate rocket;
use chrono::NaiveDateTime;
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};
use uuid::Uuid;

// list of all users
#[get("/")]
async fn list(state: &State<MyState>) -> Result<Json<Vec<User>>, BadRequest<String>> {
    let list = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(list))
}

// list all punches
#[get("/")]
async fn punches(state: &State<MyState>) -> Result<Json<Vec<PunchRecord>>, BadRequest<String>> {
    let list = sqlx::query_as("SELECT * FROM punch_records")
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
    let user_id = Uuid::new_v4();
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
    let punch =
        sqlx::query_as("INSERT INTO punch_records (user_id, in_out) VALUES ($1, $2) RETURNING *")
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
    let punch =
        sqlx::query_as("SELECT * FROM punch_records WHERE user_id = $1 ORDER BY id DESC LIMIT 1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(punch))
}

// get all punch records for a user
#[get("/<id>/punches")]
async fn get_user_punches(
    id: String,
    state: &State<MyState>,
) -> Result<Json<Vec<PunchRecord>>, BadRequest<String>> {
    let punch = sqlx::query_as("SELECT * FROM punch_records WHERE user_id = $1")
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(punch))
}

struct MyState {
    pool: PgPool,
}

// routes
#[shuttle_runtime::main]
async fn rocket(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };
    let rocket = rocket::build()
        .mount("/user", routes![retrieve, add])
        .mount("/list", routes![list, punches])
        .mount("/punch", routes![punch, last_punch, get_user_punches])
        .manage(state);

    Ok(rocket.into())
}

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "punch", rename_all = "lowercase")]
enum InOut {
    In,
    Out,
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
