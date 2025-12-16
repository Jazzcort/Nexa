# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nexa is a desktop GUI application for local AI inference, built with Tauri + SvelteKit + TypeScript. It provides a chat interface for interacting with multiple LLM providers (Ollama, Gemini) and supports MCP (Model Context Protocol) for function calling capabilities.

## Key Technologies
- **Frontend**: SvelteKit 2.x with TypeScript, TailwindCSS 4.x, Vite 6.x
- **Backend**: Tauri 2.x (Rust) with async/await via tokio
- **UI Components**: bits-ui, TipTap rich text editor with code highlighting
- **State Management**: Svelte 5 runes (`$state`, `$derived`)
- **Data Persistence**: Tauri plugin-store for chat history and settings
- **Security**: Tauri plugin-secure-storage with keyring integration

## Development Commands

### Frontend Development
- `npm run dev` - Start development server (frontend only, port 1420)
- `npm run build` - Build the frontend for production
- `npm run preview` - Preview the production build
- `npm run check` - Run Svelte type checking
- `npm run check:watch` - Run type checking in watch mode

### Tauri (Full App) Development
- `npm run tauri dev` - Start Tauri development mode (frontend + backend)
- `npm run tauri build` - Build the complete Tauri application

## Architecture Overview

### Multi-Provider LLM System

The backend implements a provider abstraction pattern for supporting multiple AI services:

**Backend Structure** (`src-tauri/src/`):
- `llm/base.rs` - Base trait for LLM providers
- `llm/ollama.rs` - Ollama-specific implementation
- `llm/gemini.rs` - Google Gemini implementation
- `llm/commands.rs` - Tauri commands exposed to frontend
- `api/gemini.rs` - HTTP client for Gemini API

**Key Tauri Commands**:
- `get_all_ollama_chat_models` - Fetches available Ollama models
- `stream_chat` - Streams chat responses from selected provider (emits events)
- `initialize_mcp_client` - Sets up MCP client for function calling
- `call_tool` - Invokes MCP tools during chat

### MCP (Model Context Protocol) Integration

The app implements MCP for function calling capabilities:

**Backend Structure** (`src-tauri/src/mcp/`):
- `client.rs` - MCP client implementation
- `connection.rs` - Process management for MCP servers
- `manager.rs` - Client lifecycle management
- `commands.rs` - Tauri commands for MCP operations
- `structs.rs` - MCP-specific data structures

**State Management**:
- `AppData` in `lib.rs` maintains a `HashMap<String, Arc<MCPClient>>` wrapped in `RwLock`
- MCP clients are lazily initialized and stored globally via Tauri's state management

### Frontend Architecture

**Type System** (`src/types/index.d.ts`):
- Discriminated unions for message types: `TextContent | FunctionCallRequestContent | FunctionCallResponseContent`
- Provider abstraction: `type Provider = "ollama" | "gemini"`
- Function call lifecycle tracking: `FunctionCallStatus` with states: `initialized`, `awaiting`, `success`, `failed`, `cancelled`

**State Management**:
- `src/states/ollamaModelState.svelte.ts` - Model selection (provider + modelId)
- `src/lib/stores/chat-history.svelte.ts` - Chat persistence using Tauri Store
  - Stores both `chatHistory` and `functionCallInfo` as separate keys
  - Uses `ReactiveFunctionCallInfo` wrapper to make function call status reactive
  - Auto-saves to `chat-history.json` via Tauri plugin-store

**Path Aliases**:
- `$states` → `./src/states`
- `$types` → `./src/types/index.d.ts`
- `$components` → `./src/components`

### Chat System Flow

1. **User Input**: TipTap editor captures rich text input
2. **Message Dispatch**: Frontend invokes `stream_chat` with provider info and message history
3. **Backend Processing**:
   - Routes to appropriate provider (Ollama/Gemini)
   - Handles streaming via async Rust iterators
   - Emits chat events with message chunks
4. **Function Calling** (if supported):
   - Model returns function call requests
   - Frontend displays pending function calls
   - User can approve/reject
   - Backend invokes MCP tools via `call_tool`
   - Results injected back into chat context
5. **State Updates**: Frontend reactively updates chat history using Svelte 5 runes
6. **Persistence**: Chat history auto-saved to Tauri Store

### Key Configuration
- **SvelteKit**: SPA mode with `adapter-static` and fallback to `index.html`
- **Vite**: Fixed port (1420) for Tauri integration with HMR
- **Tauri**: Library crate name `nexa_sveltekit_lib` with multiple crate types for cross-platform support
- **Rust Dependencies**: Uses `tauri-plugin-http` with streaming support, `tokio-util`, `async-stream` for async operations

## Important Implementation Details

- **No SSR**: Tauri enforces client-side rendering only
- **Event-Driven Streaming**: All LLM responses stream via Tauri events, not HTTP
- **Reactive Function Calls**: `FunctionCallInfo` uses `$state` for reactive status updates
- **Validated Persistence**: On load, function call info is validated against existing chat messages to prevent orphaned data
- **Multi-Provider Support**: Abstract provider interface allows easy addition of new LLM services