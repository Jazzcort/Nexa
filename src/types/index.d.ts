export interface ChatMessageWithId {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  image?: string;
  done: boolean;
}

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  image?: string;
}

export interface EmittedChatMessage {
  id: string;
  message: ChatMessage;
  done: boolean;
}

export interface ModelState {
  index: number;
  models: Model[];
}

export type Provider = "ollama" | "gemini";

export interface Model {
  provider: Provider;
  modelId: string;
}

export type ConfigSection = "general" | "apiKeys";

export interface GetItemResponse {
  data: string;
}
