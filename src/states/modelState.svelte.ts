import type { ModelState } from "$lib/type";
import { invoke } from "@tauri-apps/api/core";


const chat_models: string[] = await invoke("get_all_ollama_chat_models");

export const modelState = $state<ModelState>({
  models: chat_models,
  index: 0
})
