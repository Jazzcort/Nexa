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
  models: string[];
}
