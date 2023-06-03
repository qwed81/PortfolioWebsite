use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{get, Router};
use axum::Server;
use dashmap::DashMap;
use qwed81_dev::GitHubApi;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tower_http::services::ServeDir;

struct Article {
    rendered: String,
    refresh_time: Instant,
}

#[derive(Clone)]
struct AppState {
    cache: Arc<DashMap<String, Article>>,
    api_client: Arc<GitHubApi>,
    template: Arc<String>,
}

const CACHE_DUR: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    let template_string = fs::read_to_string("template.html")
        .await
        .expect("could not open template.html");

    let gh = GitHubApi::new("qwed81", String::from("qwed81"), String::from("min-ss"));
    let state = AppState {
        cache: Arc::new(DashMap::new()),
        api_client: Arc::new(gh),
        template: Arc::new(template_string),
    };

    let router = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/blog/:name", get(render_blog))
        .with_state(state);

    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    Server::bind(&socket_addr)
        .serve(router.into_make_service())
        .await
        .expect("could not serve");
}

async fn render_blog(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, StatusCode> {
    if let Some(article) = state.cache.get(&name) {
        let now = Instant::now();
        if now < article.refresh_time + CACHE_DUR {
            return Ok(Html(article.rendered.clone()));
        }
    }

    let refresh_time = Instant::now();
    let content = state
        .api_client
        .get_html_from_markdown(String::from("/README.md"))
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("CONTENT", &content);
    map.insert("TITLE", &name);
    map.insert("UPDATED", "10/12/2002");

    let rendered = qwed81_dev::render_to_template(&state.template, map);
    let article = Article {
        refresh_time,
        rendered: rendered.clone(),
    };

    state.cache.insert(name, article);
    Ok(Html(rendered))
}
