use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{env, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sea_orm::{
    ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait, ModelTrait, Set,
    TryIntoModel,
};

use entity::{todo, todo::Entity as Todo};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // initialize database
    let database_url = env::var("DATABASE_URL").expect("Invalid DATABASE_URL, see .env file");
    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    let state = AppState { conn: conn };
    // compose the routes
    let app = Router::new()
        .route("/todos", get(todo_index).post(create_todo))
        .route("/todos/:id", patch(update_todo).delete(delete_todo))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Hanhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(state);
    let backend_url = env::var("BACKEND_URL").expect("Invalid BACKEND_ADDR, see .env file");
    let listener = tokio::net::TcpListener::bind(&backend_url).await.unwrap();
    tracing::info!("Listening on {}", backend_url);
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}

// The query parameters for todos index
#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

async fn todo_index(
    State(state): State<AppState>,
) -> Result<Json<Vec<todo::Model>>, (StatusCode, String)> {
    Todo::find()
        .all(&state.conn)
        .await
        .map(Json)
        .map_err(internal_error)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<Json<todo::Model>, (StatusCode, String)> {
    todo::ActiveModel {
        id: ActiveValue::NotSet,
        text: Set(input.text),
        completed: Set(false),
    }
    .save(&state.conn)
    .await
    .and_then(|model| model.try_into_model().map(Json))
    .map_err(internal_error)
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<todo::Model>, (StatusCode, String)> {
    // https://www.sea-ql.org/SeaORM/docs/basic-crud/update/
    let target = Todo::find_by_id(id)
        .one(&state.conn)
        .await
        .expect(&format!("Can't find todo {}", id))
        .unwrap_or_else(|| panic!("Can't find todo {}", id));
    let mut updated: todo::ActiveModel = target.into();
    updated.completed = Set(true);
    let pear = updated.save(&state.conn).await;
    pear.and_then(|model| model.try_into_model().map(Json))
        .map_err(internal_error)
}

#[derive(Debug, Deserialize, Serialize)]
struct DeleteReponse {
    id: i32,
    text: String,
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<DeleteReponse>, (StatusCode, String)> {
    // https://www.sea-ql.org/SeaORM/docs/basic-crud/delete/
    Todo::delete_by_id(id)
        .exec(&state.conn)
        .await
        .expect(&format!("Can't find todo {}", id));
    Ok(Json(DeleteReponse {
        id: id,
        text: "Deleted".into(),
    }))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
