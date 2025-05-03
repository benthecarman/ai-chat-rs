use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use dotenv::dotenv;
use futures::StreamExt;
use std::{
    env,
    io::{self, Write},
    thread,
    time::Duration,
};

struct AppConfig {
    openai_api_key: String,
    openai_api_base: String,
    openai_model: String,
    word_display_delay_ms: u64,
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
        
        // Word display delay in milliseconds (for creating a more natural conversation flow)
        let word_display_delay_ms = env::var("WORD_DISPLAY_DELAY_MS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u64>()
            .unwrap_or(10);

        Ok(AppConfig {
            openai_api_key,
            openai_api_base,
            openai_model,
            word_display_delay_ms,
        })
    }

    fn create_openai_client(&self) -> Client<OpenAIConfig> {
        // Configure the OpenAI client with our settings
        let config = OpenAIConfig::new()
            .with_api_key(&self.openai_api_key)
            .with_api_base(&self.openai_api_base);

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
    println!("Type 'exit', 'quit', or press Ctrl+D to end the conversation.");
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

    // Start interactive chat loop with the client, model name, and display delay
    chat_loop(
        client, 
        &config.openai_model,
        config.word_display_delay_ms
    ).await?;

    Ok(())
}

/// Run the interactive chat loop
async fn chat_loop(
    client: Client<OpenAIConfig>, 
    model: &str,
    word_display_delay_ms: u64
) -> Result<()> {
    let mut input = String::new();
    // Store conversation history
    let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();
    
    // Add a system message to set the assistant's behavior
    messages.push(
        ChatCompletionRequestSystemMessageArgs::default()
            .content("You are a helpful, friendly AI assistant. Be concise and clear in your responses.")
            .build()?
            .into()
    );

    loop {
        // Clear the input buffer
        input.clear();

        // Print prompt and flush to ensure it appears before user input
        print!("You> ");
        io::stdout().flush()?;

        // Read user input, handling Ctrl+D (EOF)
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D) detected
                println!("\nCtrl+D detected. Exiting...");
                break;
            }
            Ok(_) => {
                // Normal input
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }

        // Trim whitespace
        let input = input.trim().to_string();

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

        // Add user message to history
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(input)
                .build()?
                .into()
        );

        // Build chat completion request with streaming enabled
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages.clone())
            .stream(true)
            .build()?;

        // Send request and process streaming response
        print!("AI> ");
        io::stdout().flush()?;

        let mut stream = client.chat().create_stream(request).await?;
        let mut assistant_response = String::new();

        // Process each chunk as it arrives in a word-by-word fashion
        let mut word_buffer = String::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices {
                        if let Some(content) = chat_choice.delta.content {
                            // Add to the total response
                            assistant_response.push_str(&content);
                            
                            // Process content character by character
                            for c in content.chars() {
                                word_buffer.push(c);
                                
                                // If we hit a space or punctuation, display the word
                                if c.is_whitespace() || c == '.' || c == ',' || c == '!' || c == '?' || c == ';' || c == ':' {
                                    // Print the buffer with a small delay for a more natural feel
                                    print!("{}", word_buffer);
                                    io::stdout().flush()?;
                                    
                                    // Add a configurable delay between words
                                    if word_display_delay_ms > 0 {
                                        thread::sleep(Duration::from_millis(word_display_delay_ms));
                                    }
                                    
                                    // Clear the buffer
                                    word_buffer.clear();
                                }
                            }
                            
                            // If we have characters left in the buffer, print them too
                            if !word_buffer.is_empty() {
                                print!("{}", word_buffer);
                                io::stdout().flush()?;
                            }
                        }
                    }
                }
                Err(err) => {
                    // Print any remaining buffer content before returning error
                    if !word_buffer.is_empty() {
                        print!("{}", word_buffer);
                        io::stdout().flush()?;
                    }
                    return Err(err.into());
                }
            }
        }
        
        // Print any remaining content in the buffer
        if !word_buffer.is_empty() {
            print!("{}", word_buffer);
            io::stdout().flush()?;
        }

        // Add line break after AI response
        println!();

        // Add assistant's response to conversation history
        messages.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(assistant_response)
                .build()?
                .into()
        );

        // Prevent history from growing too large
        if messages.len() > 20 {
            // Keep system message and the last 10 exchanges (20 messages)
            let system_message = messages.remove(0);
            messages = messages.split_off(messages.len().saturating_sub(19));
            messages.insert(0, system_message);
        }
    }

    Ok(())
}
