mod skill;
mod yaml;

use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{
    env, fs,
    process::{self, Command},
};

use crate::skill::{compile_all_skills_message_content, load_all_skill_files};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    let mut messages = vec![];

    let skills = load_all_skill_files();
    if !skills.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": compile_all_skills_message_content(&skills),
        }));
    }

    if args.prompt.starts_with('/') {
        let skill_name = &args.prompt.trim()[1..];
        if let Some(skill) = skills.get(skill_name) {
            messages.push(json!({
                "role": "user",
                "content": skill.body.clone(),
            }));
        }
    } else {
        messages.push(json!({
            "role": "user",
            "content": args.prompt,
        }));
    }

    let chat = client.chat();

    loop {
        let payload = json!({
            "messages": messages,
            "model": "anthropic/claude-haiku-4.5",
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "Read",
                        "description": "Read and return the contents of a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "The path to the file to read"
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "Write",
                        "description": "Write content to a file",
                        "parameters": {
                            "type": "object",
                            "required": ["file_path", "content"],
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "The path of the file to write to"
                                },
                                "content": {
                                    "type": "string",
                                    "description": "The content to write to the file"
                                }
                            }
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "Bash",
                        "description": "Execute a shell command",
                        "parameters": {
                            "type": "object",
                            "required": ["command"],
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "The command to execute"
                                }
                            }
                        }
                    }
                }
            ]
        });
        let response: Value = chat.create_byot(payload).await?;

        let mut had_tool_calls = false;
        for choice in response["choices"].as_array().unwrap() {
            let message = &choice["message"];
            messages.push(message.clone());

            if let Some(tool_calls) = message.get("tool_calls") {
                for tool_call in tool_calls.as_array().unwrap() {
                    had_tool_calls = true;
                    let kind = tool_call["type"].as_str().unwrap();

                    match kind {
                        "function" => {
                            let function_kind = tool_call["function"]["name"].as_str().unwrap();
                            let tool_call_id = tool_call["id"].as_str().unwrap();
                            match function_kind {
                                "Read" => {
                                    let read_args_json =
                                        tool_call["function"]["arguments"].as_str().unwrap();
                                    let read_args: Value =
                                        serde_json::from_str(read_args_json).unwrap();
                                    let file_path = read_args["file_path"].as_str().unwrap();
                                    let file_contents = fs::read_to_string(file_path).unwrap();

                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": file_contents,
                                    }));
                                }
                                "Write" => {
                                    let write_args_json =
                                        tool_call["function"]["arguments"].as_str().unwrap();
                                    let write_args: Value =
                                        serde_json::from_str(write_args_json).unwrap();
                                    let file_path = write_args["file_path"].as_str().unwrap();
                                    let file_contents = write_args["content"].as_str().unwrap();

                                    fs::write(file_path, file_contents).unwrap();

                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": "File written successfully",
                                    }));
                                }
                                "Bash" => {
                                    let bash_args_json =
                                        tool_call["function"]["arguments"].as_str().unwrap();
                                    let bash_args: Value =
                                        serde_json::from_str(bash_args_json).unwrap();
                                    let command = bash_args["command"].as_str().unwrap();

                                    let output = Command::new("bash")
                                        .arg("-c")
                                        .arg(command)
                                        .output()
                                        .expect("Failed to execute command");

                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": String::from_utf8_lossy(&output.stdout).to_string(),
                                    }));
                                }
                                other => {
                                    eprintln!("Unknown function kind: {}", other);
                                    panic!()
                                }
                            }
                        }
                        other => {
                            eprintln!("Unknown tool kind: {}", other);
                            panic!()
                        }
                    }
                }
            } else {
                if let Some(content) = message["content"].as_str() {
                    if !content.is_empty() {
                        println!("{}", content);
                    }
                };
            }
        }

        if !had_tool_calls {
            break;
        }
    }

    Ok(())
}
