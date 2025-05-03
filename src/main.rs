use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig};
use dotenv::dotenv;
use std::{
    env,
    io::{self, Write},
};

struct AppConfig {
    openai_api_key: String,
    openai_api_base: String,
    openai_model: String,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        // Load environment variables from .env file if it exists
        dotenv().ok();

        // Load required API key with helpful error message
        let openai_api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable is required. Please add it to your .env file or set it in your environment")?;

        // Load optional variables with defaults
        let openai_api_base =
            env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());

        Ok(AppConfig {
            openai_api_key,
            openai_api_base,
            openai_model,
        })
    }

    fn create_openai_client(&self) -> Client<OpenAIConfig> {
        // Configure the OpenAI client with our settings
        let config = OpenAIConfig::new()
            .with_api_key(self.openai_api_key.clone())
            .with_api_base(self.openai_api_base.clone());

        Client::with_config(config)
    }
}

fn print_welcome_message(model: &str) {
    println!("========================================");
    println!("🤖 Welcome to AI Chat RS 🤖");
    println!("========================================");
    println!("Connected to OpenAI API");
    println!("Using model: {}", model);
    println!();
    println!("Type your messages and press Enter to chat.");
    println!("Type 'exit' or 'quit' to end the conversation.");
    println!("========================================");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from environment variables
    let config = AppConfig::from_env()?;

    // Create the OpenAI client
    let client = config.create_openai_client();

    // Display welcome message
    print_welcome_message(&config.openai_model);

    // Start interactive chat loop
    chat_loop().await?;

    Ok(())
}

/// Run the interactive chat loop
async fn chat_loop() -> Result<()> {
    let mut input = String::new();

    loop {
        // Clear the input buffer
        input.clear();

        // Print prompt and flush to ensure it appears before user input
        print!("You> ");
        io::stdout().flush()?;

        // Read user input
        io::stdin().read_line(&mut input)?;

        // Trim whitespace
        let input = input.trim();

        // Handle exit commands
        if input.eq_ignore_ascii_case("/exit")
            || input.eq_ignore_ascii_case("/quit")
            || input.eq_ignore_ascii_case("exit")
            || input.eq_ignore_ascii_case("quit")
        {
            println!("Goodbye! Thanks for chatting.");
            break;
        }

        // Skip empty messages
        if input.is_empty() {
            continue;
        }

        // TODO: We'll implement message processing in the next step
        println!("AI> I received your message: {}", input);
    }

    Ok(())
}
