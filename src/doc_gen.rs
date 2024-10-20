use dotenvy::dotenv;
use std::fs::{read_dir, DirEntry, File};
use std::process::Command;
use std::{
    env,
    io::{Read, Write},
};

use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_base_url, set_key,
};

pub async fn doc_cleanup(doc_contents: String) -> Result<(), Box<dyn std::error::Error>> {
    let cleanup_sys_prompt = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(
            "Format the document to be a formal library document in markdown. Make sure there is consistency throughout the whole document. Have the functions that were header 2 as code blocks instead of as headers."
                .to_string(),
        ),
        name: None,
        function_call: None,
    },
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(doc_contents),
        name: None,
        function_call: None,
        },
    ];

    let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", cleanup_sys_prompt.clone())
        .create()
        .await
        .unwrap();

    let returned_message = chat_completion.choices.first().unwrap().message.clone();

    println!("cleanup entered");
    let mut docs = File::create("docs/docs_revised.md")?;

    let _ = docs.write(returned_message.content.clone().unwrap().trim().as_bytes());
    Ok(())
}

pub async fn doc_file_parse(
    mut messages: Vec<ChatCompletionMessage>,
    //mut fn_list_msgs: Vec<ChatCompletionMessage>,
    //mut function_list: Vec<String>,
    entry: DirEntry,
) -> Result<Vec<ChatCompletionMessage>, Box<dyn std::error::Error>> {
    //messages.append(&mut chat_history_setup());
    let mut file = File::open(entry.path().to_str().unwrap())?;
    println!("file: {:?}", file);
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // Make sure you have a file named `.env` with the `OPENAI_KEY` environment variable defined!

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(contents),
        name: None,
        function_call: None,
    });

    /*
    let fn_list_chat_completion = ChatCompletion::builder("gpt-3.5-turbo", fn_list_msgs.clone())
        .create()
        .await
        .unwrap();
    */
    //let returned_message = chat_completion.choices.first().unwrap().message.clone();

    let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
        .create()
        .await
        .unwrap();

    let returned_message = chat_completion.choices.first().unwrap().message.clone();

    let mut docs = File::options()
        .append(true)
        .create(true)
        .open("docs/docs.md")?;

    println!("file: {}", entry.path().to_str().unwrap());
    let returned_msg = returned_message.content.clone().unwrap();
    let returned_msg = returned_msg.trim();
    if returned_msg.contains("SKIP") {
        return Ok(messages);
    }
    let _ = docs.write(returned_msg.as_bytes());
    //messages.push(returned_message);
    Ok(messages)
}
