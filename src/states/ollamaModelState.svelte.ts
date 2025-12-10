import type { ModelState, Model } from "$types";
import { invoke } from "@tauri-apps/api/core";

// const chat_models: string[] = await invoke("get_all_ollama_chat_models");

export const modelState = $state<ModelState>({
  models: [
    { provider: "ollama", modelId: "granite3.3:8b" },
    {
      provider: "ollama",
      modelId: "qwen3:14b",
    },
    {
      provider: "gemini",
      modelId: "gemini-3-pro-preview",
    },

    {
      provider: "gemini",
      modelId: "gemini-2.5-pro",
    },
  ],
  index: 0,
});
