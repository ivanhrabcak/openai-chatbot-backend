#[macro_use] extern crate rocket;

use std::sync::{Arc};
use std::time::Duration;
use futures::lock::Mutex;
use rocket::serde::{Deserialize, json::Json};
use rocket::State;
use ttl_cache::TtlCache;
use openai::OpenAI;
use crate::auth::generate_token;

use crate::request::Response;

pub mod request;
pub mod openai;
pub mod auth;

pub struct Nothing;

type OpenAIState = State<Arc<Mutex<OpenAI>>>;
type CacheState = State<Arc<Mutex<TtlCache<String, Nothing>>>>;

const PASSWORD: &str = "GPT3MegaMind";

#[derive(Clone, Deserialize)]
pub struct Prompt {
    prompt: String
}

unsafe impl Send for Prompt {}

#[get("/")]
pub async fn index(_open_ai: &OpenAIState) -> Response<String> {
    return Response::new("Hello World!".to_string(), 200);
}

#[get("/login?<password>")]
pub async fn login(cache: &CacheState, open_ai: &OpenAIState, password: String) -> Response<String>{
    if password != PASSWORD {
        return Response::new("Wrong password".to_string(), 400);
    }

    let mut cache = cache.lock().await;
    let mut open_ai = open_ai.lock().await;

    let token = generate_token();

    (*open_ai).context_manager.crate_new_context(token.clone());

    (*cache).insert(token.clone(), Nothing, Duration::from_secs(60 * 60 * 3));

    Response::new(token.clone(), 200)
}

#[post("/chat?<token>", data = "<prompt>")]
pub async fn chat(cache: &CacheState, open_ai: &OpenAIState, prompt: Json<Prompt>, token: String) -> Response<String> {
    let prompt = prompt.clone().prompt;

    let mut cache = cache.lock().await;
    let mut open_ai = open_ai.lock().await;

    if !cache.get(&token).is_some() {
        return Response::new("Bad token.".to_string(), 401)
    }

    let response = (*open_ai).get_response(token, prompt).await;
    Response::new(response, 200)
}

#[launch]
fn rocket() -> _ {
    let API_KEY = env!("OPENAI_TOKEN");
    let openai_state = Arc::new(Mutex::new(OpenAI::new(API_KEY.to_string())));
    let token_cache: Arc<Mutex<TtlCache<String, Nothing>>> = Arc::new(Mutex::new(TtlCache::new(50)));
    rocket::build()
        .manage(token_cache)
        .manage(openai_state)
        .mount("/", routes![index, login, chat])
}