mod doc_gen;
use doc_gen::{doc_cleanup, doc_file_parse, welcome_doc};

use dotenvy::dotenv;
use openai::{
    chat::{ChatCompletionMessage, ChatCompletionMessageRole},
    set_base_url, set_key,
};
use std::fs::{read_dir, remove_file, File};
use std::path::Path;
use std::process;
use std::{
    env,
    io::{Read, Write},
};

use clap::{Arg, Command, Parser};

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
    //

    //let args = Args::parse();
    let ignore_list = vec![
        "docs.".to_string()
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

    let matches = Command::new("Knock")
        .version("1.0")
        .subcommand(
            Command::new("generate").about("Generates documention").arg(
                Arg::new("dir-path")
                    .long("dir-path")
                    .required(true)
                    .help("Directory path to project"),
            ),
        )
        .subcommand(
            Command::new("serve")
                .about("Serves files from the specified directory")
                .arg(
                    Arg::new("folder")
                        .required(true)
                        .help("Serve documentation live"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("generate", sub_m)) => {
            let dir_path: String = sub_m
                .get_one::<String>("dir-path")
                .expect("dir-path required")
                .to_owned();

            let _output = process::Command::new("sh")
                .arg("-c")
                .arg("pip install mkdocs")
                .output()
                .expect("failed to execute process");

            let _output = process::Command::new("sh")
                .arg("-c")
                .arg("mkdocs new .")
                .output()
                .expect("failed to execute process");

            //   let dir_path = args.dir_path;
            let mut dir_list = vec![dir_path];

            //let mut messages: Vec<ChatCompletionMessage> = Vec::new();
            let mut messages: Vec<ChatCompletionMessage> = chat_history_setup();
            //let fn_list_msgs: Vec<ChatCompletionMessage> = chat_history_setup_fn_list();

            println!("Files processed:");
            while let Some(dir) = dir_list.pop() {
                for entry in read_dir(dir).unwrap() {
                    if let Ok(entry) = entry {
                        let f_type = entry.file_type().unwrap();
                        if f_type.is_dir() {
                            if !ignore_list
                                .contains(&entry.file_name().to_str().unwrap().to_string())
                            {
                                dir_list.push(entry.path().to_str().unwrap().to_string());
                            }
                        }
                        if f_type.is_file() {
                            println!("{}", entry.file_name().to_str().unwrap().to_string());
                            messages = doc_file_parse(messages, entry).await?;
                        }
                    }
                }
            }
            let mut file = File::open("docs/docs.md")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let _ = doc_cleanup(contents).await;
            let _ = remove_file("docs/docs.md").unwrap();
            println!("\n\nDocumentation generated!");
            let _ = remove_file("docs/index.md");
            let mut file = File::create("docs/index.md")?;
            let _ = file.write(welcome_doc().as_bytes());

            Ok(())
        }
        Some(("serve", sub_m)) => {
            let folder = sub_m
                .get_one::<String>("folder")
                .expect("Expected folder path");
            let current_dir = env::current_dir().expect("Failed to get cwd");
            println!("Serving on http://127.0.0.1:8000/");

            let mkdocs_output = process::Command::new("mkdocs")
                .arg("serve")
                .current_dir(Path::new(folder))
                //.current_dir(&current_dir)
                .output()
                .expect("failed to execute process");

            Ok(())
        }
        _ => {
            println!("No valid subcommand provided. pier [generate | serve]");
            Ok(())
        }
    }
}
