use clap::Parser;
use dotenvy::dotenv;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Lang {
    Ja,
    En,
    Zh,
}

#[derive(clap::Parser)]
struct Args {
    #[clap(value_enum, long, short, default_value = "ja")]
    input: Lang,

    #[clap(value_enum, long, short, default_value = "en")]
    output: Lang,
}

fn main() {
    dotenv().unwrap();
    let args = Args::parse();

    let cwd = std::env::current_dir().unwrap();
    let in_path = cwd.join("_in.md");
    let out_path = cwd.join("_out.md");

    let content = std::fs::read_to_string(in_path).unwrap();
    let prompt = create_prompt(&content, args.input, args.output);
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let response = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(request(&api_key, &prompt));
    std::fs::write(out_path, response).unwrap();
}

pub fn create_prompt(content: &str, i: Lang, o: Lang) -> String {
    format!(
        "あなたは通訳者です。{}で与えられたマークダウンの内容を{}のマークダウンに翻訳してください。ただし、内容や言葉の言い回しは勝手に改変しないでください。\n\n{}",
        lang_to_prompt(i),
        lang_to_prompt(o),
        content
    )
}

fn lang_to_prompt(lang: Lang) -> String {
    match lang {
        Lang::Ja => String::from("日本語"),
        Lang::En => String::from("英語"),
        Lang::Zh => String::from("中国語"),
    }
}

pub async fn request(api_key: &str, prompt: &str) -> String {
    set_key(api_key.to_string());

    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(prompt.to_string()),
        name: None,
        function_call: None,
    }];

    let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
        .create()
        .await
        .unwrap();
    let returned_message = chat_completion.choices.first().unwrap().message.clone();

    returned_message.content.unwrap()
}
