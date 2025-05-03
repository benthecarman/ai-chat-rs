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

/// Helper function to roughly estimate token count in a conversation
/// This is a very simple approximation (4 chars ≈ 1 token)
fn estimate_token_count(messages: &[ChatCompletionRequestMessage]) -> usize {
    // For simplicity, we'll use a flat approximation
    // Each message takes about 20 tokens of overhead plus content
    
    // Count the total number of messages
    let message_count = messages.len();
    
    // Estimate the total content size by using debug formatting
    // This avoids complex type handling while getting a reasonable size estimate
    let content_size = format!("{:?}", messages).len();
    
    // Calculate approximate tokens:
    // - 4 characters per token
    // - Plus fixed overhead per message
    let token_estimate = content_size / 4 + message_count * 5;
    
    token_estimate
}
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

fn print_welcome_message(model: &str, word_delay_ms: u64) {
    println!("========================================");
    println!("🤖 Welcome to AI Chat RS 🤖");
    println!("========================================");
    println!("Connected to OpenAI API");
    println!("Using model: {}", model);
    
    // Show word display mode
    if word_delay_ms > 0 {
        println!("Word-by-word display: Enabled ({}ms delay)", word_delay_ms);
        println!("Set WORD_DISPLAY_DELAY_MS=0 to disable");
    } else {
        println!("Word-by-word display: Disabled");
        println!("Set WORD_DISPLAY_DELAY_MS to enable");
    }
    
    println!();
    println!("Type your messages and press Enter to chat.");
    println!("Commands:");
    println!("  /exit, /quit - Exit the application");
    println!("  /history, /status - Show conversation history stats");
    println!("  Ctrl+D - Exit the application");
    println!("========================================");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from environment variables
    let config = AppConfig::from_env()?;

    // Create the OpenAI client
    let client = config.create_openai_client();

    // Display welcome message
    print_welcome_message(&config.openai_model, config.word_display_delay_ms);

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

        // Handle commands
        if input.eq_ignore_ascii_case("/exit")
            || input.eq_ignore_ascii_case("/quit")
            || input.eq_ignore_ascii_case("exit")
            || input.eq_ignore_ascii_case("quit")
        {
            println!("Goodbye! Thanks for chatting.");
            break;
        } else if input.eq_ignore_ascii_case("/history") || input.eq_ignore_ascii_case("/status") {
            // Show conversation history summary
            let user_messages = messages.iter().filter(|m| match m {
                ChatCompletionRequestMessage::User(_) => true,
                _ => false
            }).count();
            
            let ai_messages = messages.iter().filter(|m| match m {
                ChatCompletionRequestMessage::Assistant(_) => true,
                _ => false
            }).count();
            
            println!("Conversation history:");
            println!("- Messages: {} total ({} user, {} AI, 1 system)", 
                     messages.len(), user_messages, ai_messages);
            println!("- Context window usage: Approximately {} tokens", 
                     estimate_token_count(&messages));
            continue;
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
        let assistant_response;

        // A completely different approach to avoid duplication:
        // Collect the entire response first, then display it word by word
        let mut full_response = String::new();
        
        // First, collect the entire response
        print!("AI> ");
        io::stdout().flush()?;
        
        // Show progress indicator if word delay is enabled (for better UX)
        let show_progress = word_display_delay_ms > 0;
        let mut progress_counter = 0;
        let progress_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices {
                        if let Some(content) = chat_choice.delta.content {
                            full_response.push_str(&content);
                            
                            // Show a spinner while collecting the response
                            if show_progress {
                                // Update the spinner every few chunks
                                progress_counter = (progress_counter + 1) % 3;
                                if progress_counter == 0 {
                                    let spinner = progress_chars[(full_response.len() / 3) % progress_chars.len()];
                                    print!("\rAI> {} ", spinner);
                                    io::stdout().flush()?;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }
        
        // Now display the full response word by word
        assistant_response = full_response.clone(); // Save for history
        
        // Clear the progress indicator if it was shown
        if show_progress {
            print!("\rAI> ");
            io::stdout().flush()?;
        }
        
        // Use a more sophisticated word splitting approach
        let mut current_word = String::new();
        
        // Get the total length once to avoid repeated counting
        let total_chars = full_response.chars().count();
        
        // Process the entire response character by character
        for (i, c) in full_response.chars().enumerate() {
            current_word.push(c);
            
            // Define word boundaries (space, punctuation, or end of text)
            let is_boundary = c.is_whitespace() || 
                              c == '.' || c == ',' || c == '!' || 
                              c == '?' || c == ';' || c == ':' ||
                              i == total_chars - 1;
            
            if is_boundary {
                // Print the word
                print!("{}", current_word);
                io::stdout().flush()?;
                
                // Add a delay if configured
                if word_display_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(word_display_delay_ms));
                }
                
                // Reset the word buffer
                current_word.clear();
            }
        }
        
        // In case there's anything left
        if !current_word.is_empty() {
            print!("{}", current_word);
            io::stdout().flush()?;
        }

        // Add line break after AI response
        println!();

        // Optional debug line to show conversation history stats
        // println!("\nSaving response to history: {} chars, {} messages in conversation", 
        //          assistant_response.len(), messages.len() + 1);
        
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
