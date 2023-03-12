use std::fs::File;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

const ROLE_SYSTEM: &'static str = "system";
const ROLE_USER: &'static str = "user";

#[tokio::main]
async fn main() {
    dioxus_desktop::launch(app)
}

fn app(cx: Scope) -> Element {

    let url_prefix = use_state(&cx, || "https://api.openai.com".to_string());
    let secret = use_state(&cx, || "".to_string());
    let system_prompt = use_state(&cx, || "假设你是一个翻译,请将用户的输入翻译成中文".to_string());
    let prompt = use_state(&cx, || "".to_string());
    let loading = use_state(&cx, || "".to_string());
    let error_msg = use_state(&cx, || "".to_string());
    let response = use_state(&cx, || ChatResponse {
        content: String::from(""),
        prompt_tokens: 0,
        completion_tokens: 0,
    });

    cx.render(rsx! {
        head {
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
            link {
                rel: "stylesheet",
                href: "bulma.min.css",
            }
            link {
                rel: "stylesheet",
                href: "//unpkg.com/@highlightjs/cdn-assets@11.7.0/styles/default.min.css",
            }
            script {
                src: "//unpkg.com/@highlightjs/cdn-assets@11.7.0/highlight.min.js"
            }
            
        }
        div { 
            class: "container is-max-desktop px-2",
            p { class: "title is-size-4 has-text-centered",
                "OpenAI测试" 
            }
            div { class: "columns",
                div { class: "column is-6",
                    input { class: "input",
                        r#type: "text",
                        value: "{url_prefix}",
                        oninput: move |evt| {
                            url_prefix.set(evt.value.clone());
                        },
                    }
                }
                div { class: "column is-6",
                    input { class: "input",
                        placeholder: "OpenAi Secret",
                        r#type: "password",
                        value: "{secret}",
                        oninput: move |evt| {
                            secret.set(evt.value.clone());
                        },
                    }
                }

            }
            div { class: "columns",
                div { class: "column",
                    p { class: "has-text-grey-light",
                        "系统prompt"
                    }
                    p { class: "control",
                        textarea { class: "textarea",
                            rows: 2,
                            value: "{system_prompt}",
                            oninput: move |evt| {
                                system_prompt.set(evt.value.clone());
                            },
                        }
                    }
                }
            }
            
            div { class: "columns",
                div { class: "column",
                    p { class: "has-text-grey-light",
                        "用户prompt"
                    }
                    p { class: "control {loading}",
                        textarea { class: "textarea",
                            value: "{prompt}",
                            oninput: move |evt| {
                                prompt.set(evt.value.clone());
                            },
                        }
                    }
                }
            }

            button { class: "button is-primary {loading}",

                onclick: move |_| cx.spawn({
                    let loading = loading.clone();
                    loading.set("is-loading".to_string());
                    
                    let url_prefix = url_prefix.clone();
                    let secret = secret.clone();
                    let system_prompt = system_prompt.clone();
                    let prompt = prompt.clone();
                    let response = response.clone();
                    let error_msg = error_msg.clone();
    

                    async move {
                        let result = request(url_prefix.current().to_string(), secret.current().to_string(), 
                            system_prompt.current().to_string(), prompt.current().to_string()).await;
                        match result {
                            Ok(res) => {
                                error_msg.set("".to_string());
                                response.set(res);
                            },
                            Err(e) => error_msg.set(e.to_string()),
                        }
                        loading.set("".to_string());
                    }
                }),
                "提交"
            }

            if !error_msg.is_empty() {
                rsx! {
                    div { class: "notification is-warning",
                        button { class: "delete",
                        onclick: move |_| {
                            error_msg.set("".to_string());
                        }},
                        "{error_msg}"
                    }
                }
            }
    
            if !response.content.is_empty() {
                rsx! {
                    article { class: "message mt-2",
                        div { class: "message-body",
                            dangerous_inner_html: "{response.content}",
                        }
                    }
                }
            }

        }

    })
}
async fn request(url_prefix:String, secret: String, system_prompt: String,
    prompt: String) -> Result<ChatResponse, Box<dyn std::error::Error>> {

    let mut messages = Vec::new();
    if !system_prompt.trim().is_empty() {
        messages.push(MessageBody {
            role: String::from(ROLE_SYSTEM),
            content: system_prompt
        })
    }
    messages.push(MessageBody { role: String::from(ROLE_USER), content: prompt });

    let client = reqwest::Client::new();
    let body = json!({
        "messages":  messages,
        "model": "gpt-3.5-turbo",
    });

    println!("body:{}", body);

    let mut authorization = "Bearer ".to_string();
    authorization.push_str(&secret);

    println!("secret: {}", &secret);
    let res = client
        .post(format!("{url_prefix}/v1/chat/completions"))
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .header("Authorization", authorization)
        .send()
        .await?
        .text()
        .await?;
    println!("result:{}", res);
    let v: Value = serde_json::from_str(&res)?;
    let error = v["error"]["message"].as_str();
    if let Some(e) = error {
        return Err(e.to_string().into());
    }
    let content = v["choices"][0]["message"]["content"].as_str().unwrap_or_else(||"").to_string();
    let prompt_tokens = v["usage"]["prompt_tokens"].as_u64().unwrap_or_else(||0);
    let completion_tokens = v["usage"]["completion_tokens"].as_u64().unwrap_or_else(||0);
    Ok(ChatResponse { content: markdown::to_html(&content), prompt_tokens, completion_tokens})
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Props, Clone)]

struct MessageBody {
    role: String,
    content: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Props, Clone)]
struct ChatResponse {
    content: String,
    prompt_tokens: u64,
    completion_tokens: u64
}
