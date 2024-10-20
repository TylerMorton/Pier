mod doc_gen;
use doc_gen::{doc_cleanup, doc_file_parse};

use dotenvy::dotenv;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_base_url, set_key,
};
use std::fs::{read_dir, File};
use std::process::Command;
use std::{
    env,
    io::Read,
};

use clap::Parser;


// Hardcoded ignore list

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    dir_path: String, //#[arg(short, long)]
                      //file_path: String
}

fn chat_history_setup() -> Vec<ChatCompletionMessage> {
    vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some("Determine if the file is a library file, if not then just respond exactly with \"SKIP\". Otherwise. Given the library file create documentation for each function in markdown format. Use the EXACT format of: function identifier with parameter types should be in h2, description then the actualy description, and additional info. Make sure there are no duplicate functions. ALWAYS end with two newlines".to_string()),
        name: None,
        function_call: None,
    },
    ]
}

fn chat_history_setup_fn_list() -> Vec<ChatCompletionMessage> {
    vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(
            "List the function names from the library file. STRICTLY list name then newline."
                .to_string(),
        ),
        name: None,
        function_call: None,
    }]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // todo timeout or the thing might never end
    let args = Args::parse();
    let ignore_list = vec![
        "benches".to_string(),
        ".git".to_string(),
        ".gitignore".to_string(),
        ".github".to_string(),
        "target".to_string(),
        "fuzz".to_string(),
        "fuzzer".to_string(),
    ];

    dotenv().unwrap();
    set_key(env::var("OPENAI_API_KEY").unwrap());
    set_base_url("https://api.openai.com/v1".to_string());

    let _output = Command::new("sh")
        .arg("-c")
        .arg("pip install mkdocs")
        .output()
        .expect("failed to execute process");

    let _output = Command::new("sh")
        .arg("-c")
        .arg("mkdocs new .")
        .output()
        .expect("failed to execute process");

    let dir_path = args.dir_path;
    let mut dir_list = vec![dir_path];

    //let mut messages: Vec<ChatCompletionMessage> = Vec::new();
    let mut messages: Vec<ChatCompletionMessage> = chat_history_setup();
    //let fn_list_msgs: Vec<ChatCompletionMessage> = chat_history_setup_fn_list();

    while let Some(dir) = dir_list.pop() {
        for entry in read_dir(dir).unwrap() {
            if let Ok(entry) = entry {
                let f_type = entry.file_type().unwrap();
                if f_type.is_dir() {
                    if !ignore_list.contains(&entry.file_name().to_str().unwrap().to_string()) {
                        dir_list.push(entry.path().to_str().unwrap().to_string());
                    }
                }
                if f_type.is_file() {
                    messages = doc_file_parse(messages, entry).await?;
                }
            }
        }
    }
    let mut file = File::open("docs/docs.md")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let _ = doc_cleanup(contents).await;

    Ok(())
}
