import type { ModelState } from "$types";
import { invoke } from "@tauri-apps/api/core";

// const chat_models: string[] = await invoke("get_all_ollama_chat_models");

export const modelState = $state<ModelState>({
  models: ["granite3.3:8b", "qwen3:14b"],
  index: 0,
});
