use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use dioxus_desktop::{WindowBuilder, Config};
use dioxus_desktop::tao::window::Icon;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

mod icon;

const ROLE_SYSTEM: &'static str = "system";
const ROLE_USER: &'static str = "user";
const SYSTEM_PROMPTS_FILE_NAME: &'static str = "system_prompts.json";
const CONFURATION_FILE_NAME: &'static str = "gui_config.json";
const OUTPUT_DIR: &'static str = "output";


#[tokio::main]
async fn main() {
    let _ = fs::create_dir(OUTPUT_DIR);

    let icon = Icon::from_rgba(Vec::from(icon::ICON_DATA), 32, 32);
    
    let mut builder = WindowBuilder::new()
            .with_title("dioxus-openai-qa-gui by fairjm");

    if let Ok(icon) = icon  {
        builder = builder.with_window_icon(Some(icon));
    }

    let cfg = Config::new().with_window(
        builder);

    dioxus_desktop::launch_cfg(app, cfg)
}

fn app(cx: Scope) -> Element {
    let configuration = use_state(&cx, || load_configuration());
    let system_prompts = use_state(&cx, || load_system_prompts());

    let setting_hide = use_state(&cx, || "is-hidden");
    let system_prompt = use_state(&cx, || "".to_string());
    let system_prompt_name = use_state(&cx, || "".to_string());
    let prompt = use_state(&cx, || "".to_string());
    let loading = use_state(&cx, || "".to_string());
    let error_msg = use_state(&cx, || "".to_string());
    let response = use_state(&cx, || ChatResponse {
        content: String::from(""),
        prompt_tokens: 0,
        completion_tokens: 0,
    });
    let system_prompt_dropdown = use_state(&cx, || "");

    cx.render(rsx! {
        style { include_str!("./bulma.min.css") }
        head {
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
            // link {
            //     rel: "stylesheet",
            //     href: "bulma.min.css",
            // }
        }
        div { 
            class: "container is-max-desktop px-2",
            nav { class: "level mt-2 mb-2",
                div { class: "level-left",
                    div { class: "level-item",
                        p { class: "title is-size-4 has-text-centered",
                            "OpenAI测试" 
                        }
                    }
                    div { class: "level-item",
                        a { class: "button is-small",
                            target: "_blank",
                            href: "https://github.com/fairjm/dioxus-openai-qa-gui",
                            span { class: "icon is-small",
                                svg { 
                                    "xmlns": "http://www.w3.org/2000/svg",
                                    "viewBox": "0 0 24 24",
                                    path { "d": "M12,2A10,10 0 0,0 2,12C2,16.42 4.87,20.17 8.84,21.5C9.34,21.58 9.5,21.27 9.5,21C9.5,20.77 9.5,20.14 9.5,19.31C6.73,19.91 6.14,17.97 6.14,17.97C5.68,16.81 5.03,16.5 5.03,16.5C4.12,15.88 5.1,15.9 5.1,15.9C6.1,15.97 6.63,16.93 6.63,16.93C7.5,18.45 8.97,18 9.54,17.76C9.63,17.11 9.89,16.67 10.17,16.42C7.95,16.17 5.62,15.31 5.62,11.5C5.62,10.39 6,9.5 6.65,8.79C6.55,8.54 6.2,7.5 6.75,6.15C6.75,6.15 7.59,5.88 9.5,7.17C10.29,6.95 11.15,6.84 12,6.84C12.85,6.84 13.71,6.95 14.5,7.17C16.41,5.88 17.25,6.15 17.25,6.15C17.8,7.5 17.45,8.54 17.35,8.79C18,9.5 18.38,10.39 18.38,11.5C18.38,15.32 16.04,16.16 13.81,16.41C14.17,16.72 14.5,17.33 14.5,18.26C14.5,19.6 14.5,20.68 14.5,21C14.5,21.27 14.66,21.59 15.17,21.5C19.14,20.16 22,16.42 22,12A10,10 0 0,0 12,2Z" 
                                    }
                                }
                            }
                            span {
                                "GitHub"
                            }
                        }
                    }
                }
            }

            button { class: "button is-white is-small",
                onclick: move |_| {
                    if setting_hide.is_empty() {
                        setting_hide.set("is-hidden");
                    } else {
                        setting_hide.set("");
                    }
                },
                span { class: "icon has-text-light",
                    svg { 
                        "xmlns": "http://www.w3.org/2000/svg",
                        "viewBox": "0 0 24 24",
                        path { "d": "M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.21,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.21,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.67 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z" 
                        }
                    }
                }
                span {
                    "设置"
                }
            }

            div { class: "columns {setting_hide}",
                div { class: "column is-6",
                    input { class: "input",
                        r#type: "text",
                        value: "{configuration.url_prefix}",
                        oninput: move |evt| {
                            let conf = Configuration {url_prefix: evt.value.clone(), secret: configuration.current().secret.clone()};
                            save_configuration(&conf);
                            configuration.set(conf);
                        },
                    }
                }
                div { class: "column is-6",
                    input { class: "input",
                        placeholder: "OpenAi Secret",
                        r#type: "password",
                        value: "{configuration.secret}",
                        oninput: move |evt| {
                            let conf = Configuration {url_prefix: configuration.current().url_prefix.clone(), secret: evt.value.clone()};
                            save_configuration(&conf);
                            configuration.set(conf);
                        },
                    }
                }

            }
            div { class: "columns",
                div { class: "column",
                    nav { class: "level mb-1",
                        div { class: "level-left",
                            div { class: "level-item",
                                p { class: "has-text-grey-light",
                                    "系统prompt"
                                }
                            }
                        }
                    }
                    
                    div { class: "block mb-1",
                        div { class: "dropdown {system_prompt_dropdown}",
                            div { class: "dropdown-trigger",
                                button { class: "button",
                                    "aria-haspopup": true,
                                    "aria-controls": "dropdown-menu",
                                    onclick: move |_| {
                                        if system_prompt_dropdown.current().is_empty() {
                                            system_prompt_dropdown.set("is-active");
                                        } else {
                                            system_prompt_dropdown.set("");
                                        }
                                    },
                                    span {
                                        "prompt列表"
                                    },
                                    span { class: "icon is-small",
                                        svg { xmlns: "http://www.w3.org/2000/svg",
                                            "viewBox": "0 0 24 24",
                                            path { d: "M7,10L12,15L17,10H7Z",
                                            }
                                        }
                                }
                            }
                        }

                            div { class: "dropdown-menu",
                                id: "dropdown-menu",
                                role: "menu",
                                div { class: "dropdown-content",
                                    a {class: "dropdown-item py-0",
                                        onclick: move |_| {
                                            system_prompt_dropdown.set("");
                                        },
                                        "关闭"
                                    }
                                    hr { class:"dropdown-divider" }
                                    if system_prompts.is_empty() {
                                        rsx! {
                                            div { class: "dropdown-item",
                                                p {
                                                    "没有system prompts"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "dropdown-item",
                                        div {class: "columns is-multiline",
                                        system_prompts.iter().map(|e| {
                                            rsx!(
                                                div {class: "column",
                                                    span { class: "tag is-primary is-light",
                                                        onclick: move |_| {
                                                            system_prompt_name.set(e.name.clone());
                                                            system_prompt.set(e.content.clone());
                                                            system_prompt_dropdown.set("");
                                                        },
                                                        "{e.name}" 
                                                    
                                                        button { class: "delete is-small",
                                                            onclick: move |_| {
                                                                system_prompts.with_mut(|v| {
                                                                    if let Some(p) = v.iter().position(|value| value.name.eq(&e.name)) {
                                                                        v.remove(p);
                                                                    }
                                                                });
                                                                save_system_prompts(&*system_prompts.current().clone());
                                                            }
                                                        }
                                                    }
                                                })
                                            })
                                        }
                                    }
                                            
                                }
                            }
                        }
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
                    div { class: "level {save_button_attr(system_prompt)} mt-1",
                        div { class: "level-left",
                            div { class: "level-item",
                                input { class: "input",
                                    placeholder: "prompt名(重名将会覆盖已有的内容)",
                                    r#type: "text",
                                    value: "{system_prompt_name}",
                                    oninput: move |evt| {
                                        system_prompt_name.set(evt.value.clone())
                                    },
                                }
                            }
                            div { class: "level-item",
                                button { class: "button is-primary",
                                    disabled: "{system_prompt_name.is_empty()}",
                                    onclick: move |_| {
                                        system_prompts.with_mut(|e| {
                                            if let Some(v) = e.iter_mut()
                                                .find(|p| p.name.eq(&*system_prompt_name.current())) {
                                                    v.content = system_prompt.current().clone().to_string();
                                                } else {
                                                    e.push(SystemPrompt {name: system_prompt_name.current().clone().to_string(), content: system_prompt.current().clone().to_string()});
                                                }
                                        });
                                        save_system_prompts(&*system_prompts.current().clone());
                                    },
                                    "保存prompt"
                                }
                            }
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

            button { class: "button is-primary my-1 {loading}",
                disabled: "{request_button_disable(configuration, system_prompt, prompt)}",
                onclick: move |_| cx.spawn({
                    let loading = loading.clone();
                    loading.set("is-loading".to_string());
                    
                    let configuration = configuration.clone();
                    let system_prompt = system_prompt.clone();
                    let prompt = prompt.clone();
                    let response = response.clone();
                    let error_msg = error_msg.clone();
    

                    async move {
                        let result = request(configuration.current().url_prefix.clone(), configuration.current().secret.clone(), 
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

            if request_button_disable(configuration, system_prompt, prompt) {
                rsx! {
                    div { class: "notification is-warning",
                        "请检查url前缀, openAI密钥是否为空, system prompt和用户prompt必须有一个不为空"
                    }
                }
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
            content: system_prompt.clone()
        })
    }
    messages.push(MessageBody { role: String::from(ROLE_USER), content: prompt.clone() });

    let client = reqwest::Client::new();
    let body = json!({
        "messages":  messages,
        "model": "gpt-3.5-turbo",
    });

    println!("body:{}", body);

    let mut authorization = "Bearer ".to_string();
    authorization.push_str(&secret);

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

    let mut path = PathBuf::new();
    let mut file_name = current_date_time();
    file_name.push_str(".txt");
    path.push(OUTPUT_DIR);
    path.push(file_name);
    write_plain_data(path.as_path(),&format!(
        "system prompt:{}\nuser prompt:{}\n\nanswer:{}\nprompt_tokens:{} completion_tokens:{}\n\nfull body:{}",
        system_prompt, prompt, content, prompt_tokens, completion_tokens, res
    ));
    Ok(ChatResponse { content: markdown::to_html(&content), prompt_tokens, completion_tokens})
}

fn request_button_disable(config : &Configuration, system_prompt: &str, user_prompt: &str) -> bool {
    config.secret.is_empty() || config.url_prefix.is_empty() || (system_prompt.is_empty() && user_prompt.is_empty())
}

fn save_button_attr(system_prompt: &str) -> String{
    if system_prompt.trim().is_empty() {
        "is-hidden".to_string()
    } else {
        "".to_string()
    }
}

fn current_date_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d_%H_%M_%S").to_string()
}

fn load_configuration() -> Configuration {
    load_data(CONFURATION_FILE_NAME)
}

fn save_configuration(config : &Configuration) {
    write_data(CONFURATION_FILE_NAME, config);
}


fn load_system_prompts() -> Vec<SystemPrompt> {
    load_data(SYSTEM_PROMPTS_FILE_NAME)
}

fn save_system_prompts(prompts : &Vec<SystemPrompt>) {
    write_data(SYSTEM_PROMPTS_FILE_NAME, prompts);
}

fn load_data<P, T>(path: P) -> T 
where 
    P: AsRef<Path>,
    T: Default + DeserializeOwned, 
{
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| T::default()),
        Err(_) => {
            return T::default();
        }
    }
}

fn write_data<P, T>(path: P, data: &T) 
where 
    P: AsRef<Path>,
    T:Serialize 
{
    let _ = fs::write(path, serde_json::to_string_pretty(data).unwrap_or("".to_string()));
}

fn write_plain_data<P>(path: P, data: &str) 
where 
    P: AsRef<Path>
{
    let _ = fs::write(path, data);
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Props, Clone)]
struct SystemPrompt {
    name: String,
    content: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Props, Clone)]
struct Configuration {
    url_prefix: String,
    secret: String
}

impl Default for Configuration {
    fn default() -> Self {
        Self { url_prefix: "https://api.openai.com".to_string(), secret: Default::default() }
    }
}