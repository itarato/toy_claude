use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, fs, process};

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

    #[allow(unused_variables)]
    let response: Value = client
        .chat()
        .create_byot(json!({
            "messages": [
                {
                    "role": "user",
                    "content": args.prompt
                }
            ],
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
                }
            ]
        }))
        .await?;

    for choice in response["choices"].as_array().unwrap() {
        let message = &choice["message"];

        if let Some(content) = message["content"].as_str() {
            if !content.is_empty() {
                println!("{}", content);
            }
        };

        if let Some(tool_calls) = message.get("tool_calls") {
            for tool_call in tool_calls.as_array().unwrap() {
                let kind = tool_call["type"].as_str().unwrap();

                match kind {
                    "function" => {
                        let function_kind = tool_call["function"]["name"].as_str().unwrap();
                        match function_kind {
                            "Read" => {
                                let read_args_json =
                                    tool_call["function"]["arguments"].as_str().unwrap();
                                let read_args: Value =
                                    serde_json::from_str(read_args_json).unwrap();
                                let file_path = read_args["file_path"].as_str().unwrap();
                                let file_contents = fs::read_to_string(file_path).unwrap();
                                println!("{}", file_contents);
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
        }
    }

    Ok(())
}
