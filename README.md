# AI Chat RS

A lightweight CLI chat application that connects to OpenAI's API, built in Rust.

> **Note:** This project was created by following an AI-assisted tutorial. Both the implementation and this documentation were developed through collaborative sessions with AI tools. The project serves as a practical demonstration of using AI assistance for software development, showcasing how AI can guide developers through the process of building functional applications from scratch.

## Features

- Interactive CLI interface for chatting with OpenAI models
- Streaming responses with configurable word-by-word display
- Conversation history management
- Simple command system for basic operations

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ai-chat-rs.git
   cd ai-chat-rs
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Create a `.env` file with your OpenAI API key:
   ```
   OPENAI_API_KEY=your_api_key_here
   ```

## Usage

Run the application:

```bash
cargo run --release
```

### Configuration

Configure the application through environment variables in your `.env` file:

- `OPENAI_API_KEY` - Your OpenAI API key (required)
- `OPENAI_API_BASE` - API endpoint (default: https://api.openai.com/v1)
- `OPENAI_MODEL` - Model to use (default: gpt-4)
- `WORD_DISPLAY_DELAY_MS` - Delay between words for streaming display (default: 10ms, set to 0 to disable)

### Commands

- `/exit` or `/quit` - Exit the application
- `/history` or `/status` - Show conversation statistics
- Ctrl+D - Exit the application
