#[macro_use]
extern crate rocket;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::US::Mountain;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Request};
use rocket::response::status::BadRequest;
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use shuttle_runtime::CustomError;
use sqlx::Row;
use sqlx::{Executor, FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;
use sqlx::{postgres::PgQueryResult, Error as SqlxError};
use anyhow::Error;
use std::string::ToString;

#[derive(FromForm)]
struct LoginForm {
    user_token: String,
}

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    message: String,
    redirect: Option<String>,
}

#[post("/login", data = "<login_form>")]
async fn login(
    cookies: &CookieJar<'_>,
    login_form: Form<LoginForm>,
    state: &State<MyState>,
) -> Result<Json<LoginResponse>, Custom<String>> {
    // In a real-world scenario, validate the user_token against your data store
    println!("login form {:?}", login_form.user_token);
    if is_valid_token(&login_form.user_token, state).await {
        println!("cloning cookie {}", &login_form.user_token);
        cookies.add_private(Cookie::new("user_token", login_form.user_token.clone()));
        Ok(Json(LoginResponse {
            success: true,
            message: "Login sucessful".to_string(),
            // redirect to where they were going
            redirect: Some(cookies.get("redirect").map(|c| c.value().to_string()).unwrap_or("/home".to_string())),
        }))
    } else {
        Ok(Json(LoginResponse {
            success: false,
            message: "Login failed".to_string(),
            redirect: None,
        }))
    }
}

#[get("/login_form")]
async fn login_form() -> Template {
    Template::render("loginform", context! {})
}

struct Authenticated;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let db_pool = match req.rocket().state::<MyState>() {
            Some(state) => state,
            None => return rocket::request::Outcome::Error((Status::InternalServerError, ())),
        };

        let cookies = req.cookies();

        if let Some(cookie) = cookies.get_private("user_token") {
            if is_valid_token(cookie.value(), db_pool.into()).await {
                rocket::request::Outcome::Success(Authenticated)
            } else {
                rocket::request::Outcome::Error((Status::Unauthorized, ()))
            }
        } else {
            rocket::request::Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

async fn is_valid_token(token: &str, state: &State<MyState>) -> bool {
    // Check if the token matches the static value first
    if token == "mysupersecrettoken" {
        println!("Token matched the static secret token.");
        return true;
    }

    // Proceed with the database check if it's not the static token
    let result = sqlx::query("SELECT EXISTS (SELECT 1 FROM users WHERE user_id = $1)")
        .bind(token)
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(record) => {
            // Depending on the SQLx version and database used, the way to extract the EXISTS result might vary.
            // Assuming `record` here is a Row and you're querying a PostgreSQL database.
            // The correct column index or name should be used based on your actual query's return.
            let exists: bool = record.try_get(0).unwrap_or(false); // Use column index if column name is not available
            if exists {
                println!("A record with the token was found in the database.");
            }
            exists
        }
        Err(_) => {
            println!("Error querying the database.");
            false
        }
    }
}

#[get("/error")]
fn error_page() -> Template {
    let mut context = HashMap::new();
    context.insert("message", "The page you're looking for doesn't exist.");
    Template::render("error", &context)
}

// Example of a custom error catcher
#[catch(404)]
fn not_found() -> Custom<Template> {
    let mut context = HashMap::new();
    context.insert("message", "Resource was not found.");
    Custom(Status::NotFound, Template::render("error", &context))
}

#[catch(500)]
fn internal_error() -> Custom<Template> {
    let mut context = HashMap::new();
    context.insert("message", "Internal Server Error");
    Custom(Status::NotFound, Template::render("error", &context))
}

#[get("/")]
async fn id_list(
    _auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Json<Vec<UserId>>, BadRequest<String>> {
    match _auth {
        Some(_) => {
            // User is authenticated
            let list = sqlx::query_as("SELECT user_id, name FROM users")
                .fetch_all(&state.pool)
                .await
                .map_err(|e| BadRequest(e.to_string()))?;
            Ok(Json(list))
        }
        None => {
            // User is not authenticated, provide a message and render the login form.
            Err(BadRequest("You are not authenticated.".to_string()))
        }
    }
}

#[get("/userlist")]
async fn userlist(
    _auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Template, Status> {
    // Use Status for a more general error type.
    match _auth {
        Some(_) => {
            // User is authenticated
            println!("user is authenticated");
            match user_list(state).await {
                Ok(users) => {
                    let mut context = HashMap::new();
                    context.insert("users", users.into_inner());
                    Ok(Template::render("userlist", &context))
                }
                Err(e) => {
                    eprintln!("Failed to get user list: {:?}", e);
                    let mut context = HashMap::new();
                    context.insert("message", "Failed to get the user list.");
                    Ok(Template::render("error", &context)) // Provide a context with a message.
                }
            }
        }
        None => {
            // User is not authenticated, provide a message and render the login form.
            println!("user is not authenticated");
            let mut ctx = HashMap::new();
            ctx.insert("message", "You are not authenticated.");
            println!(
                "attempting to load login with message {}",
                ctx.get("message").unwrap_or(&"Unknown error") // Use `unwrap_or` to avoid panicking.
            );
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// get status /status only if auth
#[get("/status")]
async fn status(
    _auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Template, Status> {
    match _auth {
        Some(_) => {
            // User is authenticated
            user_statuses(state, false).await
        },
        None => {
            // User is not authenticated, provide a message and render the login form.
            let mut ctx = HashMap::new();
            ctx.insert("message", "You are not authenticated.");
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// only in status
#[get("/status/in")]
async fn status_in(
    _auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Template, Status> {
    match _auth {
        Some(_) => {
            // User is authenticated
            user_statuses(state, true).await
        },
        None => {
            // User is not authenticated, provide a message and render the login form.
            let mut ctx = HashMap::new();
            ctx.insert("message", "You are not authenticated.");
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// get users that are currently in

async fn user_statuses(state: &State<MyState>, filter_in: bool) -> Result<Template, Status> {
    // appropriate template
    let template_name = if filter_in { "users_in" } else { "user_statuses" };
    // appropriate query
    let sql_query = if filter_in {
        r#"
            SELECT
                u.name,
                p.in_out,
                p.punch_time as last_punch_time
            FROM
                users u
            INNER JOIN
                punches p ON u.user_id = p.user_id
            INNER JOIN
                (
                    SELECT
                        user_id,
                        MAX(punch_time) as max_punch_time
                    FROM
                        punches
                    WHERE
                        punch_time >= NOW() - INTERVAL '24 HOURS'
                        AND in_out = 'in'  -- Filter for 'in' status
                    GROUP BY
                        user_id
                ) as latest_punch ON p.user_id = latest_punch.user_id AND 
                p.punch_time = latest_punch.max_punch_time
            ORDER BY
                u.name, p.punch_time DESC;
        "#
    } else {
    r#"
        SELECT
            u.name,
            p.in_out,
            p.punch_time as last_punch_time
        FROM
            users u
        INNER JOIN
            punches p ON u.user_id = p.user_id
        INNER JOIN
            (
                SELECT
                    user_id,
                    MAX(punch_time) as max_punch_time
                FROM
                    punches
                WHERE
                    punch_time >= NOW() - INTERVAL '24 HOURS'
                GROUP BY
                    user_id
            ) as latest_punch ON p.user_id = latest_punch.user_id AND 
            p.punch_time = latest_punch.max_punch_time
        ORDER BY
            u.name, p.punch_time DESC;
        "#
    };
    
    let mut user_statuses: Vec<UserStatus> = sqlx::query_as::<_, UserStatus>(sql_query)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // sort by time
    user_statuses.sort_by(|a, b| b.last_punch_time.cmp(&a.last_punch_time));

    println!("user_statuses: {:?}", user_statuses);

    let formatted_user_statuses: Vec<_> = user_statuses
        .into_iter()
        .map(|status| {
            // Assume the NaiveDateTime is in UTC, then convert to local time and format
            let utc_datetime: DateTime<Utc> =
                DateTime::<Utc>::from_utc(status.last_punch_time, Utc);
            let mountain_datetime = utc_datetime.with_timezone(&Mountain);
            let formatted_time = mountain_datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            // Return the status with the formatted time
            UserStatusDisplay {
                name: status.name,
                in_out: status.in_out,
                last_punch_time: formatted_time, // This will now be a String
            }
        })
        .collect();

    let context = context! { user_statuses: formatted_user_statuses };
    Ok(Template::render(template_name, context))
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
    // use extractor
    let user = sqlx::query_as("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(user))
}

#[post("/bulk", data = "<data>")]
async fn add_bulk(data: Json<Vec<User>>, state: &State<MyState>) -> Result<Json<Vec<User>>, BadRequest<String>> {
    let mut users = Vec::new();

    for user_data in data.into_inner() {
        // generate if user_id not provided
        let user_id = match &user_data.user_id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };
        let user = sqlx::query_as("INSERT INTO users (name, email, user_id, dept_id) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(&user_data.name)
            .bind(&user_data.email)
            .bind(user_id.to_string())
            .bind(user_data.dept_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| BadRequest(e.to_string()))?;
        users.push(user);
    }

    Ok(Json(users))
}

// create a new user
#[post("/", data = "<data>")]
async fn add(data: Json<User>, state: &State<MyState>) -> Result<Json<User>, BadRequest<String>> {
    // generate if user_id not provided
    let user_id = match &data.user_id {
        Some(id) => id.to_string(),
        None => Uuid::new_v4().to_string(),
    };
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
    // if let Err(e) = initialize_db(&pool).await {
    //     eprintln!("Database initialization failed: {:?}", e);
    // } else {
    //     eprintln!("Database initialized using connection pool: {:?}", pool);
    // }

    let state = MyState { pool };
    let rocket = rocket::build()
        .attach(Template::fairing())
        .mount("/user", routes![retrieve, add, add_bulk, status, status_in])
        .mount("/list", routes![user_list, punches_list]) // comment out to prevent listing
        .mount("/punch", routes![punch, last_punch, get_user_punches])
        .mount("/static", FileServer::from("static"))
        .mount("/id", routes![id_list])
        .mount(
            "/",
            routes![index, home, login, login_form, userlist, register, error_page],
        )
        .register("/", catchers![not_found, internal_error])
        .manage(state);

    Ok(rocket.into())
}

// Define the function to initialize your database
async fn initialize_db(pool: &PgPool) -> Result<(), Error> {
    let init_sql = include_str!("../db_files/init.sql");
    let mut transaction = pool.begin().await.map_err(Error::msg)?;

    for command in split_sql_commands(init_sql) {
        let command = command.trim();
        if !command.is_empty() {
            transaction.execute(command).await.map_err(Error::msg)?;
        }
    }

    transaction.commit().await.map_err(Error::msg)?;
    Ok(())
}

// split sql commands
fn split_sql_commands(init_sql: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut in_block = false;
    println!("init_sql: {}", init_sql);

    for line in init_sql.lines() {
        if line.contains("DO $$") {
            in_block = true;
        }

        if in_block {
            // Add the line to the current command and continue until the block ends
            current_command.push_str(line);
            current_command.push('\n');
            println!("current_command: {}", current_command);
            
            if line.contains("$$") {
                in_block = false;
                commands.push(current_command.clone());
                current_command.clear();
            }
        } else {
            // If not in a block, handle line by line as separate commands
            if line.trim().ends_with(';') {
                current_command.push_str(line);
                println!("current_command: {}", current_command);
                commands.push(current_command.clone());
                current_command.clear();
            } else {
                current_command.push_str(line);
                current_command.push('\n');
            }
        }
    }

    // Add any remaining command
    if !current_command.trim().is_empty() {
        commands.push(current_command);
    }

    commands
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
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
    dept_id: Option<i32>,
}

#[derive(Deserialize, Serialize, FromRow)]
struct UserId {
    user_id: String,
    name: Option<String>,
}

// TODO: add Option<dept_id> to UserStatus
#[derive(sqlx::FromRow, Serialize, Debug)]
struct UserStatus {
    name: String,
    in_out: InOut,
    last_punch_time: NaiveDateTime,
}
#[derive(sqlx::FromRow, Serialize)]
struct UserStatusDisplay {
    name: String,
    in_out: InOut,
    last_punch_time: String, // Now it's a String to hold the formatted date
}

#[derive(sqlx::FromRow, Serialize)]
struct UserInOutStatus {
    name: String,
    last_in_time: Option<NaiveDateTime>, // These are optional to handle 'NULL'
    last_out_time: Option<NaiveDateTime>,
}
