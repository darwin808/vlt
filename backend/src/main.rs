use std::{env, net::SocketAddr};

use argon2::{
    password_hash::{PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    cookie_secure: bool,
}

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.1 });
        (self.0, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Auth extractor
// ---------------------------------------------------------------------------

struct AuthUser {
    id: String,
    email: String,
    encryption_salt: String,
}

/// Minimal trait so we can pull AppState from the state parameter.
trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for AppState {
    fn from_ref(input: &AppState) -> Self {
        input.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract cookies from the request headers.
        let jar = CookieJar::from_headers(&parts.headers);

        let session_id = jar
            .get("vlt_session")
            .map(|c| c.value().to_string())
            .ok_or_else(|| AppError(StatusCode::UNAUTHORIZED, "Not authenticated".into()))?;

        // Look up session and join with user.
        let row = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT u.id, u.email, u.encryption_salt, s.expires_at \
             FROM sessions s JOIN users u ON u.id = s.user_id \
             WHERE s.id = ?",
        )
        .bind(&session_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| AppError(StatusCode::UNAUTHORIZED, "Invalid session".into()))?;

        // Check expiry — expires_at is stored as an ISO-8601 string.
        let now = chrono_now();
        if row.3 < now {
            // Expired — clean it up.
            let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
                .bind(&session_id)
                .execute(&app_state.db)
                .await;
            return Err(AppError(StatusCode::UNAUTHORIZED, "Session expired".into()));
        }

        Ok(AuthUser {
            id: row.0,
            email: row.1,
            encryption_salt: row.2,
        })
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SignupRequest {
    email: String,
    password: String,
    encryption_salt: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateSecretRequest {
    name: String,
    #[serde(rename = "type")]
    secret_type: String,
    encrypted_value: String,
    iv: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    url: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct SecretRow {
    id: String,
    user_id: String,
    name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    secret_type: String,
    encrypted_value: String,
    iv: String,
    username: String,
    url: String,
    created_at: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return current UTC time as ISO-8601 string matching SQLite's datetime().
fn chrono_now() -> String {
    // Format: "YYYY-MM-DD HH:MM:SS"
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let secs_per_day: u64 = 86400;
    let days = now / secs_per_day;
    let time_of_day = now % secs_per_day;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    days += 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn expires_at_7_days() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 7 * 86400;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn make_session_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build(("vlt_session", value))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(secure)
        .path("/")
        .max_age(time::Duration::days(7))
        .build()
}

fn make_removal_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build(("vlt_session", ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(secure)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build()
}

fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.')
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignupRequest>,
) -> Result<(CookieJar, Json<serde_json::Value>), AppError> {
    if !is_valid_email(&body.email) {
        return Err(AppError(StatusCode::BAD_REQUEST, "Invalid email".into()));
    }
    if body.password.len() < 10 {
        return Err(AppError(
            StatusCode::BAD_REQUEST,
            "Password must be at least 10 characters".into(),
        ));
    }

    // Check for existing user
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE email = ?")
            .bind(&body.email)
            .fetch_optional(&state.db)
            .await?;

    if existing.is_some() {
        return Err(AppError(
            StatusCode::CONFLICT,
            "Email already registered".into(),
        ));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)?
        .to_string();

    let user_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, encryption_salt) VALUES (?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.encryption_salt)
    .execute(&state.db)
    .await?;

    // Create session
    let session_id = uuid::Uuid::new_v4().to_string();
    let expires = expires_at_7_days();

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(&user_id)
        .bind(&expires)
        .execute(&state.db)
        .await?;

    let jar = jar.add(make_session_cookie(session_id, state.cookie_secure));

    let resp = serde_json::json!({
        "id": user_id,
        "email": body.email,
        "encryption_salt": body.encryption_salt,
    });

    Ok((jar, Json(resp)))
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<serde_json::Value>), AppError> {
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, email, password_hash, encryption_salt FROM users WHERE email = ?",
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, email, stored_hash, encryption_salt) = row.ok_or_else(|| {
        AppError(StatusCode::UNAUTHORIZED, "Invalid email or password".into())
    })?;

    // Verify password
    let parsed_hash = PasswordHash::new(&stored_hash)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError(StatusCode::UNAUTHORIZED, "Invalid email or password".into()))?;

    // Create session
    let session_id = uuid::Uuid::new_v4().to_string();
    let expires = expires_at_7_days();

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(&user_id)
        .bind(&expires)
        .execute(&state.db)
        .await?;

    let jar = jar.add(make_session_cookie(session_id, state.cookie_secure));

    let resp = serde_json::json!({
        "id": user_id,
        "email": email,
        "encryption_salt": encryption_salt,
    });

    Ok((jar, Json(resp)))
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<serde_json::Value>), AppError> {
    if let Some(cookie) = jar.get("vlt_session") {
        let session_id = cookie.value();
        let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(&state.db)
            .await;
    }

    let jar = jar.add(make_removal_cookie(state.cookie_secure));
    Ok((jar, Json(serde_json::json!({ "ok": true }))))
}

async fn me(user: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "encryption_salt": user.encryption_salt,
    }))
}

async fn list_secrets(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<SecretRow>>, AppError> {
    let secrets: Vec<SecretRow> = sqlx::query_as(
        "SELECT id, user_id, name, type, encrypted_value, iv, username, url, created_at \
         FROM secrets WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(secrets))
}

async fn create_secret(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateSecretRequest>,
) -> Result<(StatusCode, Json<SecretRow>), AppError> {
    let valid_types = ["password", "api_key", "env_var", "ssh_key", "note"];
    if !valid_types.contains(&body.secret_type.as_str()) {
        return Err(AppError(
            StatusCode::BAD_REQUEST,
            "Invalid secret type".into(),
        ));
    }

    let secret_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO secrets (id, user_id, name, type, encrypted_value, iv, username, url) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&secret_id)
    .bind(&user.id)
    .bind(&body.name)
    .bind(&body.secret_type)
    .bind(&body.encrypted_value)
    .bind(&body.iv)
    .bind(&body.username)
    .bind(&body.url)
    .execute(&state.db)
    .await?;

    let secret: SecretRow = sqlx::query_as(
        "SELECT id, user_id, name, type, encrypted_value, iv, username, url, created_at \
         FROM secrets WHERE id = ?",
    )
    .bind(&secret_id)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(secret)))
}

async fn delete_secret(
    State(state): State<AppState>,
    user: AuthUser,
    Path(secret_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM secrets WHERE id = ? AND user_id = ?")
        .bind(&secret_id)
        .bind(&user.id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError(StatusCode::NOT_FOUND, "Secret not found".into()));
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./vlt.db?mode=rwc".into());
    let frontend_dir =
        env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend/build".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let cookie_secure: bool = env::var("COOKIE_SECURE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(true);

    // Database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Enable WAL mode and foreign keys
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("Failed to set WAL mode");
    sqlx::query("PRAGMA foreign_keys=ON;")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        db: pool,
        cookie_secure,
    };

    // API routes
    let api = Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/secrets", get(list_secrets).post(create_secret))
        .route("/secrets/:id", delete(delete_secret))
        .with_state(state);

    // SPA fallback: serve index.html for any non-API, non-file route.
    let index_path = format!("{}/index.html", &frontend_dir);
    let spa = ServeDir::new(&frontend_dir).fallback(ServeFile::new(&index_path));

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(spa);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
