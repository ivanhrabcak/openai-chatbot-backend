use std::collections::HashMap;

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub const INITIAL_CONTEXT: &str = concat!("The following is a conversation with an AI assistant. The assistant is helpful, creative, clever, and very friendly.\n\n",
                              "Human: Hello, who are you?\n",
                              "AI: I am an AI created by OpenAI. How can I help you today?\n",
                              "Human: ");

pub const ENGINE: &str = "davinci";

#[derive(Hash, PartialEq, Eq)]
pub struct Context {
    pub conversation: String
}

impl Context {
    pub fn new(conversation: String) -> Self { Self { conversation } }
}

#[derive(Serialize, Deserialize)]
struct CompletionParams {
    pub prompt: String,
    pub max_tokens: i64,
    pub top_p: f32, // A number called nucleus sampling, where the model considers the results of the tokens with top_p probability mass.
    pub temperature: f32, // The higher the value, for example, 0.9, the more “creative” the completion text will be.
    pub n: i32, // An integer that specifies the number of completions to generate for each prompt.
    pub stream: bool, // A boolean value to determine if partial progress should be streamed back.
    pub echo: bool, // A boolean value that tells the API to ech back the prompt along with the completion
    pub stop: String, // A string or array that acts as a delimiter to tell the API will stop generating further tokens.
    pub frequency_penalty: f32
}

impl Default for CompletionParams {
    fn default() -> Self {
        CompletionParams {
            prompt: INITIAL_CONTEXT.to_string(),
            max_tokens: 150,
            temperature: 0.8,
            top_p: 1.0,
            stream: false,
            n: 1,
            stop: "\n".to_string(),
            echo: true,
            frequency_penalty: 0.3
        }
    }
}

pub struct ContextManager {
    pub contexts: HashMap<String, Context>
}

impl ContextManager {
    pub fn new() -> Self { Self { contexts: HashMap::new() } }

    pub fn crate_new_context(&mut self, user_token: String) {
        let initial_context = Context::new(INITIAL_CONTEXT.to_string());
        self.contexts.insert(user_token, initial_context);
    }
    
    pub fn add_to_context(&mut self, user_token: String, s: String) {
        let ctx = self.contexts.get(&user_token).unwrap(); // tokens have to be valid to get through to this function

        let mut conversation = ctx.conversation.clone();
        conversation += &s;
        
        let new_ctx = Context::new(conversation);

        self.contexts.insert(user_token, new_ctx);
    }

    pub fn get_context(&self, user_token: String) -> String {
        self.contexts.get(&user_token).unwrap().conversation.clone()
    }

    pub fn delete_context(&mut self, user_token: String) {
        self.contexts.remove(&user_token);
    }
}

pub struct OpenAI {
    pub context_manager: ContextManager,
    pub api_key: String,
    pub client: Client,
}

impl OpenAI {
    pub fn new(api_key: String) -> Self { 
        Self { context_manager: ContextManager::new(), api_key, client: Client::new() } 
    }

    pub async fn get_response(&mut self, user_token: String, user_prompt: String) -> String {
        println!("{}", user_prompt);
        self.context_manager.add_to_context(user_token.clone(), user_prompt.clone());

        let context = self.context_manager.get_context(user_token.clone());

        let mut completion_params = CompletionParams::default();
        completion_params.prompt = format!("{}\nAI:", context);

        let completion_params = match serde_json::to_string(&completion_params) {
            Ok(params) => params,
            Err(_) => return "Serialization Error!".to_string()
        };

        println!("{}", completion_params);

        let endpoint = format!("https://api.openai.com/v1/engines/{}/completions", ENGINE);
        let response = self.client.post(endpoint)
            .bearer_auth(self.api_key.clone())
            .header("Content-Type", "application/json")
            .body(completion_params)

            .send().await;

        match response {
            Ok(response) => {
                let response_text = response.text().await.unwrap();
                println!("{}", response_text);
                let response: HashMap<String, Value> = serde_json::from_str(&*response_text).unwrap();

                let choices = response.get("choices");

                if let Some(choices) = choices {
                    match choices {
                        Value::Null => {
                            println!("Null response json!");
                            "The AI returned no response.".to_string()
                        },
                        Value::Array(array) => {
                            let ai_output_map = array[0].as_object().unwrap();

                            // we know the 'text' key is string and present if we get to this line of code
                            let ai_output_text = ai_output_map.get("text").unwrap().as_str().unwrap();

                            let new_conversation = ai_output_text.to_string() + "\nHuman: ";

                            self.context_manager.contexts.insert(user_token.clone(), Context::new(new_conversation));

                            ai_output_text.split("\n")
                                .last().unwrap()
                                .replace("AI: ", "")
                        }
                        _ => {
                            println!("Unexpected response!");
                            "The AI returned no response.".to_string()
                        }
                    }
                }
                else {
                    println!("Null choices!");
                    "The AI returned no response.".to_string()
                }
            },
            Err(_) => "Request building error!".to_string()
        }
    }

    pub fn create_context(&mut self, user_token: String) {
        self.context_manager.contexts.insert(user_token, Context::new(INITIAL_CONTEXT.to_string()));
    }
}