# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri + SvelteKit + TypeScript application that provides a chat interface for interacting with Ollama models. The project combines a Rust backend (Tauri) with a SvelteKit frontend, configured as a Single Page Application (SPA).

## Key Technologies
- **Frontend**: SvelteKit 2.x with TypeScript, TailwindCSS 4.x, Vite 6.x
- **Backend**: Tauri 2.x (Rust)
- **UI Components**: bits-ui, custom components with TailwindCSS
- **Rich Text**: TipTap editor with code highlighting support
- **State Management**: Svelte 5 runes (`$state`)

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

## Project Structure

### Frontend Architecture
- **Routes**: Standard SvelteKit routing in `src/routes/`
  - `/` - Landing page with navigation
  - `/chat` - Main chat interface with streaming support
  - `/config` - Configuration page
- **Components**: Located in `src/components/` and `src/lib/components/ui/`
  - UI components follow a modular pattern with TypeScript exports
  - TipTap editor integration for rich text chat messages
- **State Management**: `src/states/` - Svelte 5 runes for reactive state
  - `ollamaModelState.svelte.ts` - Model selection and management
- **Types**: `src/types/index.d.ts` - TypeScript definitions

### Backend Architecture (Rust/Tauri)
- **Entry Point**: `src-tauri/src/main.rs` calls `nexa_sveltekit_lib::run()`
- **LLM Module**: `src-tauri/src/llm/` - Handles model interactions
- **Tauri Commands**: Exposed via `invoke()` for frontend-backend communication
  - `stream_chat` - Streams chat responses from Ollama models
- **Events**: Uses Tauri's event system for real-time chat streaming

### Key Configuration
- **SvelteKit**: Configured as SPA with `adapter-static` and fallback to `index.html`
- **Path Aliases**:
  - `$states` → `./src/states`
  - `$types` → `./src/types/index.d.ts`
  - `$components` → `./src/components`
- **Vite**: Optimized for Tauri with fixed port (1420) and HMR support

## Chat System Architecture

The chat interface implements real-time streaming:
1. User input via TipTap editor
2. Frontend invokes `stream_chat` Tauri command
3. Backend streams responses via Tauri events
4. Frontend updates chat history reactively using Svelte 5 runes
5. Messages stored with unique IDs and completion status

## Development Notes

- Uses Svelte 5 with runes syntax (`$state`, `$derived`)
- TailwindCSS 4.x with custom color highlighting for code blocks
- Tauri prevents SSR - all rendering happens client-side
- Model selection managed through reactive state pattern
- Event-driven architecture for real-time chat streaming