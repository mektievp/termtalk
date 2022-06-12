use actix_web::web::Bytes;
use awc::ws;
use colored::*;
use futures_util::{SinkExt as _, StreamExt as _};
use serde_json::json;
use std::io::Write;
use std::{env, io, process, str, thread};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
extern crate colored;
use serde::Deserialize;

extern crate redis;

#[derive(Deserialize)]
struct Message {
    text: String,
    color: String,
}

async fn login_request(
    username: &str,
    password: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let post_body = json!({
        "username": username,
        "password": password
    });
    let post_body_str = post_body.to_string();
    client
        .post(format!(
            "{}://{}/login",
            env::var("PROTOCOL").unwrap(),
            env::var("HOST").unwrap()
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_body_str)
        .send()
        .await
}

async fn register_request(
    username: &str,
    email: &str,
    password: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let post_body = json!({
        "username": username,
        "email": email,
        "password": password
    });
    let post_body_str = post_body.to_string();
    client
        .post(format!(
            "{}://{}/register",
            env::var("PROTOCOL").unwrap(),
            env::var("HOST").unwrap()
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_body_str)
        .send()
        .await
}

async fn login_prompt() {
    let jwt_token: &awc::error::HeaderValue;
    let mut attempts: usize = 0;

    loop {
        if attempts > 2 {
            println!(
                "Made {} login attempts and that's too many. termtalk-cli will exit now",
                attempts
            );
            process::exit(0);
        }
        print!("Username >> ");
        let mut username = String::with_capacity(32);
        let _ = io::stdout().flush();
        if io::stdin().read_line(&mut username).is_err() {
            return;
        }
        let username_newline_stripped = username.strip_suffix("\n").unwrap();

        let password = rpassword::prompt_password("Password >> ").unwrap();

        match login_request(&username_newline_stripped, &password).await {
            Ok(resp) => {
                attempts += 1;
                if resp.status() != 200 {
                    println!("Something went wrong, check your input and try again");

                    log::debug!("This is what went wrong: {:?}", resp.text().await);
                    continue;
                }
                jwt_token = resp.headers().get("Authorization").unwrap();
                env::set_var("TERMTALK_CLI_JWT_TOKEN", jwt_token.to_str().unwrap());
                break;
            }
            Err(_) => {
                attempts += 1;
                log::error!("Something went wrong while trying to login. Please try again");
            }
        };
    }
}

async fn register_prompt() {
    println!("Input registration details >> ");

    let mut register_username = String::with_capacity(32);

    print!("Username >> ");

    let _ = io::stdout().flush();
    if io::stdin().read_line(&mut register_username).is_err() {
        return;
    }

    let mut register_email = String::with_capacity(32);

    print!("Email >> ");

    let _ = io::stdout().flush();
    if io::stdin().read_line(&mut register_email).is_err() {
        return;
    }

    let register_password = rpassword::prompt_password("Password >> ").unwrap();

    let register_username_stripped = register_username.strip_suffix("\n").unwrap();
    let register_email_stripped = register_email.strip_suffix("\n").unwrap();

    match register_request(
        &register_username_stripped,
        &register_email_stripped,
        &register_password,
    )
    .await
    {
        Ok(resp) => {
            if resp.status() != 201 {
                println!("Registration failed. termtalk-cli will exit now");
                process::exit(0);
            }
            println!("User {} with email {} was successfully registered. Make sure to login to use termtalk-cli", register_username_stripped, register_email_stripped);
        }
        Err(e) => println!("e: {:?}", e),
    };
}

fn set_default_env_vars() {
    match env::var("RUST_LOG") {
        Ok(_) => {}
        Err(_) => env::set_var("RUST_LOG", "info"),
    };

    match env::var("HOST") {
        Ok(_) => {}
        Err(_) => env::set_var("HOST", "http://localhost:8080"),
    };
}

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();
    set_default_env_vars();
    env_logger::init();

    println!("termtalk-cli has started\n\n");
    loop {
        println!("What would you like to do?:\n1. Login\n2. Register");
        let mut option = String::with_capacity(32);
        if io::stdin().read_line(&mut option).is_err() {
            return;
        }

        let option_input = option.strip_suffix("\n").unwrap();
        match option_input {
            "1" => {
                login_prompt().await;
                break;
            }
            "2" => {
                register_prompt().await;
            }
            val => println!("{} is not a valid input, try again", val),
        };
    }

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

    let (resp, mut ws) = awc::Client::new()
        .ws(format!("ws://{}/connect", env::var("HOST").unwrap()))
        .set_header(
            "Authorization",
            format!("Bearer {}", env::var("TERMTALK_CLI_JWT_TOKEN").unwrap()),
        )
        .connect()
        .await
        .unwrap();

    if resp.status() != 101 {
        println!("Could not connect to termtalk-api server. Exiting now");
        process::exit(0);
    }
    println!("{}", "Successfully connected to the termtalk-api\n".green());

    println!("Choose from one of the following commands:\n\n/whoami - username and current channel\n/r /rooms - list existing rooms\n/o /online - which users are online\n/d /direct (user)\n/j /join (room)\n/h /here - list users in current room\n/w /whisper (user)\n/h /help - view this list of commands\n/e /exit - quit termtalk-cli\n\n");
    println!("You are currently in the ");

    // run blocking terminal input reader on separate thread
    thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }

        match cmd.as_str().strip_suffix("\n").unwrap() {
            "/u" | "/users" => {
                println!("{}", "Users Online:".green());
            }
            _ => {}
        }
        cmd_tx.send(cmd).unwrap();
    });

    loop {
        select! {
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        let txt_from_bytes: &str = str::from_utf8(&txt).unwrap();
                        let msg: Message = serde_json::from_str(txt_from_bytes).unwrap();
                        println!("{}", msg.text.color(msg.color));
                    }

                    Ok(ws::Frame::Ping(_)) => {
                        ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                    }
                    Ok(ws::Frame::Close(close_reason)) => {
                        let unwrapped_close_msg: String = close_reason.unwrap().description.unwrap();
                        println!("{}", format!("Server closed connection because {}\nWill exit now", unwrapped_close_msg).green());
                        process::exit(0);
                    }

                    _ => {}
                }
            }

            Some(cmd) = cmd_rx.next() => {
                let cmd_stripped = cmd.strip_suffix("\n").unwrap();
                if cmd_stripped.is_empty() {
                    continue;
                }

                ws.send(ws::Message::Text(cmd_stripped.into())).await.unwrap();
            }

            else => {}
        }
    }
}
