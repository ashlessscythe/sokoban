#[macro_use]
extern crate rocket;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::US::Mountain;
use rocket::http::Method;
use rocket::{
    form::Form,
    fs::FileServer,
    http::{Cookie, CookieJar, Status},
    request::{FromRequest, Request},
    response::{
        status::{BadRequest, Custom},
        Redirect,
    },
    serde::json::Json,
    State,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::postgres::Postgres;
use sqlx::{FromRow, PgPool, Row};

use std::{collections::HashMap, string::ToString};

use uuid::Uuid;
mod func;

// add db-check route
#[get("/db-check")]
async fn db_check(pool: &State<MyState>) -> String {
    dotenv::dotenv().ok();
    let simualte_db_offline = std::env::var("SIMULATE_DB_OFFLINE").is_ok();
    println!("simualte_db_offline: {:?}", simualte_db_offline);

    if simualte_db_offline {
        return "0: Database is offline".to_string();
    }
    match sqlx::query("SELECT 1")
        .fetch_one(&pool.pool)
        .await
    {
        Ok(_) => "1: Database is online".to_string(),
        Err(_) => "0: Database is offline".to_string(),
    }
}

#[derive(FromForm)]
struct LoginForm {
    user_token: String,
    device_id: String,
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
    if is_valid_token(&login_form.user_token, &login_form.device_id, state).await {
        println!("cloning cookie {}", &login_form.user_token);
        cookies.add_private(Cookie::new("user_token", login_form.user_token.clone()));
        cookies.add_private(Cookie::new("device_id", login_form.device_id.clone()));
        Ok(Json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
            // redirect to where they were going
            redirect: Some(
                cookies
                    .get("redirect")
                    .map(|c| c.value().to_string())
                    .unwrap_or("/home".to_string()),
            ),
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

        if let Some(user_token_cookie) = cookies.get_private("user_token") {
            if let Some(device_id_cookie) = cookies.get_private("device_id") {
                if is_valid_token(user_token_cookie.value(), device_id_cookie.value(), db_pool.into()).await {
                    rocket::request::Outcome::Success(Authenticated)
                } else {
                    rocket::request::Outcome::Error((Status::Unauthorized, ()))
                }
            } else {
                rocket::request::Outcome::Error((Status::Unauthorized, ()))
            }
        } else {
            rocket::request::Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

async fn is_valid_token(token: &str, device_id: &str, state: &State<MyState>) -> bool {
    // Check if the token matches the static value first
    if token == "mysecrettoken" {
        println!("Token matched the static secret token.");
        return true;
    }

    println!("Looking for token: {}", token);
    // Proceed with the database check if it's not the static token
    let user_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)")
        .bind(token)
        .fetch_one(&state.pool)
        .await;

    println!("Looking for device_id: {}", device_id);
    let device_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM auth_devices WHERE device_id = $1)")
        .bind(device_id)
        .fetch_one(&state.pool)
        .await;

    // return boolean true if token exists in db and device_id exists in auth_devices
    match (user_exists, device_exists) {
        (Ok(user_row), Ok(device_row)) => user_row.get(0) && device_row.get(0),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("Failed to check token or device_id: {:?}", e);
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
            Err(BadRequest("User or device not authenticated.".to_string()))
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
            ctx.insert("message", "User or device not authenticated.");
            println!(
                "attempting to load login with message {}",
                ctx.get("message").unwrap_or(&"Unknown error") // Use `unwrap_or` to avoid panicking.
            );
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// get status /status only if auth
#[get("/status_list")]
async fn status_list(
    _auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Template, Status> {
    match _auth {
        Some(_) => {
            // User is authenticated
            println!("getting template for status list");
            get_status_list(state).await
        }
        None => {
            // User is not authenticated, provide a message and render the login form.
            let mut ctx = HashMap::new();
            ctx.insert("message", "User or device not authenticated.");
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// list of latest punches
async fn get_status_list(state: &State<MyState>) -> Result<Template, Status> {
    // appropriate template
    let template_name = "status_list";

    println!("template_name: {:?}", template_name);

    // appropriate query
    let sql_query = (r#"
        SELECT
            u.user_id,
            u.name,
            p.in_out,
            p.punch_time as last_punch_time,
            COALESCE(d.name, 'No Department') as dept_name
        FROM
            users u
        INNER JOIN
            punches p ON u.user_id = p.user_id
        LEFT JOIN
            departments d ON u.dept_id = d.id
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
        "#);

    let mut user_statuses: Vec<UserStatus> = sqlx::query_as::<Postgres, UserStatus>(sql_query)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            eprintln!("Failed to get user statuses: {:?}", e);
            Status::InternalServerError
        })?;

    // sort by time
    user_statuses.sort_by(|a, b| b.last_punch_time.cmp(&a.last_punch_time));

    println!("user_statuses: {:?}", user_statuses);

    // temp id

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
                temp_id: func::generate_temp_id(&status.user_id),
                name: status.name,
                in_out: status.in_out,
                last_punch_time: formatted_time, // This will now be a String
                dept_name: status.dept_name,
                drill_id: None,
                found: None,
            }
        })
        .collect();

    println!("formatted_user_statuses: {:?}", formatted_user_statuses);

    let context = context! { user_statuses: formatted_user_statuses };
    println!("template_name: {:?}", template_name);
    Ok(Template::render(template_name, context))
}

// only in status
#[get("/checklist")]
async fn checklist(
    _b_auth: Option<Authenticated>,
    state: &State<MyState>,
) -> Result<Template, Status> {
    match _b_auth {
        Some(_) => {
            // User is authenticated
            println!("getting template for in status");
            get_checklist(state).await
        }
        None => {
            // User is not authenticated, provide a message and render the login form.
            let mut ctx = HashMap::new();
            ctx.insert("message", "User or device not authenticated.");
            Ok(Template::render("loginform", &ctx)) // Ensure you return `Ok` here.
        }
    }
}

// get users that are currently in
async fn get_checklist(state: &State<MyState>) -> Result<Template, Status> {
    let template_name = "checklist";
    let current_drill_id = func::get_drill_id(None);
    println!("current_drill_id: {:?}", current_drill_id);

    // Step 1: Fetch user statuses
    let user_statuses = sqlx::query_as::<_, UserNameDept>(
       "SELECT
                u.user_id, -- We'll need this to link to the 'in' status, but won't use it in the struct
                u.name,
                COALESCE(d.name, 'No Department') as dept_name
            FROM
                users u
            LEFT JOIN
                departments d ON u.dept_id = d.id
            WHERE
                EXISTS (
                    SELECT 1
                    FROM punches p
                    WHERE
                        p.user_id = u.user_id
                        AND p.in_out = 'in'
                        AND p.punch_time >= NOW() - INTERVAL '24 HOURS'
                        AND p.punch_time = (
                            SELECT MAX(punch_time)
                            FROM punches
                            WHERE user_id = u.user_id
                            AND punch_time >= NOW() - INTERVAL '24 HOURS'
                        )
                )
            ORDER BY
                u.name;
            "
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| Status::InternalServerError)?;
    println!("user_statuses: {:?}", user_statuses);

    // Step 2: Ensure a checklist_status entry for each user for the current drill_id
    for user_status in &user_statuses {
        sqlx::query(
            "INSERT INTO checklist_status (user_id, drill_id, found) VALUES ($1, $2, false)
             ON CONFLICT (user_id, drill_id) DO NOTHING",
        )
        .bind(&user_status.user_id)
        .bind(current_drill_id)
        .execute(&state.pool)
        .await
        .map_err(|_| Status::InternalServerError)?;
    }
    println!(
        "checklist_status entries created for current drill_id: {:?}",
        current_drill_id
    );

    // Fetch the actual checklist statuses with the found status
    let checklist_statuses: Vec<FoundStatusUpdate> =
        sqlx::query_as("SELECT user_id, drill_id, found FROM checklist_status WHERE drill_id = $1")
            .bind(current_drill_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                eprintln!("Failed to get checklist statuses: {:?}", e);
                Status::InternalServerError
            })?;
    println!("checklist_statuses: {:?}", checklist_statuses);

    fn convert_to_map(checklist_statuses: Vec<FoundStatusUpdate>) -> HashMap<String, bool> {
        checklist_statuses
            .into_iter()
            .map(|cs| (cs.user_id, cs.found))
            .collect()
    }

    let checklist_status_map = convert_to_map(checklist_statuses);
    println!("checklist_status_map: {:?}", checklist_status_map);

    // Step 3: Now fetch the joined user statuses and checklist status
    let formatted_user_statuses: Vec<_> = user_statuses
        .into_iter()
        .map(|status| {
            // get status from map
            let found_status = *checklist_status_map.get(&status.user_id).unwrap_or(&false);
            println!(
                "found_status: {:?} for id {:?}",
                found_status, &status.user_id
            );

            // Return the status with the formatted time
            UserChecklistDisplay {
                temp_id: func::generate_temp_id(&status.user_id),
                name: status.name,
                dept_name: status.dept_name,
                drill_id: current_drill_id,
                found: found_status,
            }
        })
        .collect();

    // Finally, create the context and render the template
    println!("template_name: {:?}", template_name);
    let context = context! { user_statuses: formatted_user_statuses };
    Ok(Template::render(template_name, context))
}

// build hashmap of ids from user table
async fn build_user_id_hashmap(
    state: &State<MyState>,
) -> Result<HashMap<String, String>, BadRequest<Json<String>>> {
    let user_ids_result = sqlx::query_as::<_, UserOnly>("SELECT user_id FROM users")
        .fetch_all(&state.pool)
        .await;

    let user_ids = match user_ids_result {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("Failed to get user list: {:?}", e);
            return Err(BadRequest(Json("Failed to get user list".to_string())));
        }
    };

    let user_id_hashmap = user_ids
        .into_iter()
        .map(|user| {
            let hashed_id = func::generate_temp_id(&user.user_id); // Make sure this function exists and is accessible
            (hashed_id, user.user_id)
        })
        .collect::<HashMap<String, String>>();

    Ok(user_id_hashmap)
}

// update checklist status
#[post("/update-found-status", format = "json", data = "<found_status>")]
async fn update_found_status(
    state: &State<MyState>,
    found_status: Json<FoundStatusUpdate>,
) -> Result<Json<FoundStatusUpdate>, Status> {
    // print drill_id
    let default_drill_id = Some(func::get_drill_id(None));
    println!("default_drill_id: {:?}", default_drill_id);

    // build hashmap of ids from user table
    let user_id_hashmap = build_user_id_hashmap(state)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let temp_id = found_status.user_id.clone();
    let original_user_id = if let Some(user_id) = user_id_hashmap.get(temp_id.as_str()) {
        user_id
    } else {
        return Err(Status::NotFound);
    };

    // update status
    println!("original_user_id: {:?}", original_user_id);
    println!(
        "drill_id: {:?}",
        found_status.drill_id.unwrap_or(func::get_drill_id(None))
    );
    println!("found: {:?}", found_status.found);

    let result = sqlx::query(
        "INSERT INTO checklist_status (user_id, drill_id, found) VALUES ($1, $2, $3)
            ON CONFLICT (user_id, drill_id) DO UPDATE SET found = $3",
    )
    .bind(original_user_id)
    .bind(
        found_status
            .drill_id
            .unwrap_or_else(|| func::get_drill_id(None)),
    )
    .bind(found_status.found)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        eprintln!("SQL Error: {:?}", e);
        Status::InternalServerError
    })?;

    println!(
        "Number of rows inserted or updated: {:?}",
        result.rows_affected()
    );

    println!("result: {:?}", result);
    Ok(Json(found_status.into_inner()))
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
    let list: Vec<PunchWithUser> = sqlx::query_as("SELECT * FROM punches_with_user")
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
async fn add_bulk(
    data: Json<Vec<User>>,
    state: &State<MyState>,
) -> Result<Json<Vec<User>>, BadRequest<String>> {
    let mut users = Vec::new();

    for user_data in data.into_inner() {
        // generate if user_id not provided
        let user_id = match &user_data.user_id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };
        let user = sqlx::query_as(
            "INSERT INTO users (name, email, user_id, dept_id) VALUES ($1, $2, $3, $4) RETURNING *",
        )
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
    let user_id = data
        .user_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, user_id, dept_id) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&data.name)
    .bind(&data.email)
    .bind(user_id.to_string())
    .bind(&data.dept_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(user))
}

// punch user in or out
#[post("/<user_id>", data = "<punch_data>")]
async fn punch(
    user_id: String,
    punch_data: Json<PunchRecord>,
    state: &State<MyState>,
) -> Result<Json<PunchRecord>, BadRequest<String>> {
    let device_id = punch_data.device_id.as_deref().unwrap_or("No-Device-Id"); // or use a default value that makes sense in your context

    let punch = sqlx::query_as::<_, PunchRecord>(
        "INSERT INTO punches (user_id, in_out, device_id) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&user_id)
    .bind(&punch_data.in_out)
    .bind(device_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(punch))
}


// get number of punches for user
#[get("/<id>/count")]
async fn count_punches(
    id: String,
    state: &State<MyState>,
) -> Result<Json<i64>, BadRequest<String>> {
    let count = sqlx::query("SELECT COUNT(*) FROM punches WHERE user_id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    Ok(Json(count.get(0)))
}

// get users last punch
#[get("/<id>/last_punch")]
async fn last_punch(
    id: String,
    state: &State<MyState>,
) -> Result<Json<PunchRecord>, BadRequest<String>> {
    let punch = sqlx::query_as::<Postgres, PunchRecord>(
        "SELECT * FROM punches WHERE user_id = $1 AND punch_time > NOW() - INTERVAL '24 HOURS' ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match punch {
        Ok(Some(punch)) => Ok(Json(punch)),
        Ok(None) => Ok(Json(PunchRecord {
            in_out: InOut::None,
            device_id: None,
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
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("database_url: {:?}", database_url);

    // construct pool
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    // state & static files
    let state = MyState { pool };
    let static_files_dir = std::env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "static".into());

    // debug
    println!("Current working directory: {:?}", std::env::current_dir());

    let origins_str =
        std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:8000/".into());
    let origins = origins_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();
    println!("origins are {:?}", &origins);

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&origins), // Adjust according to your needs
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS");

    // rocket
    let rocket = rocket::build()
        .attach(cors)
        .attach(Template::fairing())
        .mount("/user", routes![retrieve, add, add_bulk])
        // .mount("/list", routes![user_list, punches_list]) // comment out for deployed
        .mount("/punch", routes![punch, last_punch, count_punches, get_user_punches])
        .mount("/status", routes![status_list, checklist, update_found_status])
        .mount("/static", FileServer::from(static_files_dir))
        // .mount("/id", routes![id_list])
        .mount(
            "/",
            routes![index, db_check, home, login, login_form, register, error_page],
        )
        .register("/", catchers![not_found, internal_error])
        .manage(state);

    match rocket.launch().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
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

#[derive(FromRow, Serialize)]
struct PunchWithUser {
    user_name: String,
    in_out: InOut,
    device_id: Option<String>,
    punch_time: Option<NaiveDateTime>,
}

#[derive(FromRow, Serialize, Deserialize)]
struct PunchRecord {
    in_out: InOut,
    device_id: Option<String>,
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
struct UserOnly {
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct UserNameDept {
    user_id: String,
    name: String,
    dept_name: String,
}

#[derive(Deserialize, Serialize, FromRow)]
struct UserId {
    user_id: String,
    name: Option<String>,
}

// TODO: add Option<dept_id> to UserStatus
#[derive(sqlx::FromRow, Debug)]
struct UserStatus {
    user_id: String,
    name: String,
    in_out: InOut,
    last_punch_time: NaiveDateTime,
    dept_name: String,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
struct UserChecklistDisplay {
    temp_id: String,
    name: String,
    dept_name: String,
    drill_id: i32,
    found: bool,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
struct UserStatusDisplay {
    temp_id: String,
    name: String,
    in_out: InOut,
    last_punch_time: String, // Now it's a String to hold the formatted date
    dept_name: String,
    drill_id: Option<i32>,
    found: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct FoundStatusUpdate {
    user_id: String,
    drill_id: Option<i32>,
    found: bool,
}