
import { invoke } from "@tauri-apps/api/core";


const chat_models = await invoke("get_all_ollama_chat_models");

export const modelState = $state({
  models: chat_models,
  index: 0
})
