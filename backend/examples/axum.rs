#![allow(clippy::unwrap_used, clippy::print_stderr)] // ok for example

use axum::Extension;
use axum::body::Body;
use axum::http::Request;
use axum::middleware::from_fn;
use axum::{Router, middleware::Next, response::IntoResponse, routing::get};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::task_local;

trait MyState: Send + Sync + Debug {
    fn jobs(&self) -> Box<dyn JobService>;

    fn tasks(&self) -> &[Arc<Box<dyn Task>>];
}

#[derive(Debug, Clone)]
struct GlobalState {
    jobs: NoopJobService,
    tasks: Vec<Arc<Box<dyn Task>>>,
}

task_local! {
    pub static USER: String;
}

#[derive(Debug, Clone)]
struct UserState {
    jobs: UserJobService,
    global: Arc<Box<dyn MyState>>,
}

impl MyState for GlobalState {
    fn jobs(&self) -> Box<dyn JobService> {
        Box::new(self.jobs.clone())
    }

    fn tasks(&self) -> &[Arc<Box<dyn Task>>] {
        &self.tasks
    }
}

impl MyState for UserState {
    fn jobs(&self) -> Box<dyn JobService> {
        Box::new(self.jobs.clone())
    }

    fn tasks(&self) -> &[Arc<Box<dyn Task>>] {
        self.global.tasks()
    }
}

trait JobService {
    fn register(&self);
}

#[derive(Debug, Clone)]
struct NoopJobService;

impl JobService for NoopJobService {
    fn register(&self) {
        eprintln!("do nothing");
    }
}

#[derive(Debug, Clone)]
struct UserJobService {
    user: String,
}

impl JobService for UserJobService {
    fn register(&self) {
        eprintln!("register job for user {}", self.user);
    }
}

trait Task: Send + Sync + Debug {
    fn execute(&self);
}

#[derive(Debug, Clone)]
struct PrintTask {
    message: String,
}

impl Task for PrintTask {
    fn execute(&self) {
        eprintln!("{} -> {}", self.message, USER.with(Clone::clone));
    }
}

#[tokio::main]
async fn main() {
    let state: Arc<Box<dyn MyState>> = Arc::new(Box::new(GlobalState {
        jobs: NoopJobService,
        tasks: vec![Arc::new(Box::new(PrintTask {
            message: "Hello, World!".into(),
        }))],
    }));

    // attach middleware to the route so it runs after the router's state is available
    let app = Router::new()
        .route("/", get(hello).layer(from_fn(swap_state)))
        .layer(Extension(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello(Extension(state): Extension<Arc<Box<dyn MyState>>>) -> impl IntoResponse {
    state.jobs().register();

    state.tasks().iter().for_each(|task| task.execute());
}

async fn swap_state(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    let user = "User123".to_string();

    // create a UserJobService instance and replace the request state
    let new_state: Arc<Box<dyn MyState>> = Arc::new(Box::new(UserState {
        jobs: UserJobService { user: user.clone() },
        global: req
            .extensions()
            .get::<Arc<Box<dyn MyState>>>()
            .unwrap()
            .clone(),
    }));

    // insert/overwrite the state in request extensions so `Extension<T>` extractor sees it
    req.extensions_mut().insert(new_state);

    // continue the request, scoped
    USER.scope(user, next.run(req)).await
}
