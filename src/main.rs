use anyhow as ah;
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
    content: String,
    refresh_time: Instant,
}

#[derive(Clone)]
struct AppState {
    cache: Arc<DashMap<String, Article>>,
    api_client: Arc<GitHubApi>,
    template: Arc<String>,
    index_template: Arc<String>,
}

const CACHE_DUR: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() {
    let template_string = fs::read_to_string("template.html")
        .await
        .expect("could not open template.html");

    let index_template_string = fs::read_to_string("index.html")
        .await
        .expect("could not open index.html");

    // load the client for the repository qwed81/blog-md
    let gh = GitHubApi::new("qwed81", String::from("qwed81"), String::from("blog-md"));

    let state = AppState {
        cache: Arc::new(DashMap::new()),
        api_client: Arc::new(gh),
        template: Arc::new(template_string),
        index_template: Arc::new(index_template_string),
    };

    let router = Router::new()
        .nest_service("/static", ServeDir::new("public"))
        .route("/", get(get_index))
        .route("/:name", get(render_blog))
        .with_state(state);

    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on 8080");
    Server::bind(&socket_addr)
        .serve(router.into_make_service())
        .await
        .expect("could not serve");
}

async fn load_and_cache(state: &AppState, name: String) -> ah::Result<String> {
    if let Some(article) = state.cache.get(&name) {
        let now = Instant::now();
        if now < article.refresh_time + CACHE_DUR {
            return Ok(article.content.clone());
        }
    }

    let refresh_time = Instant::now();
    let content = state
        .api_client
        .get_html_from_markdown(format!("/{}.md", name))
        .await?;

    let article = Article {
        content: content.clone(),
        refresh_time,
    };

    state.cache.insert(name.clone(), article);
    Ok(content)
}

async fn get_index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let projects_content = load_and_cache(&state, String::from("projects"))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contact_content = load_and_cache(&state, String::from("contact"))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("PROJECTS", &projects_content);
    map.insert("CONTACT", &contact_content);

    let rendered = qwed81_dev::render_to_template(&state.index_template, map);
    Ok(Html(rendered))
}

async fn render_blog(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let content = load_and_cache(&state, name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // load the title from the h1 tag
    let title = qwed81_dev::get_h1_title(&content).unwrap_or_else(|| String::from("qwed81.dev"));

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("CONTENT", &content);
    map.insert("TITLE", &title);

    let rendered = qwed81_dev::render_to_template(&state.template, map);
    Ok(Html(rendered))
}
