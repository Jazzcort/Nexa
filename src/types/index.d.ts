export type ChatMessageWithId = ChatMessage & {
  id: string;
  done: boolean;
};
// {
//   id: string;
//   role: "user" | "assistant" | "system" | "function";
//   content: ChatMessageContent;
//   image?: string;
//   done: boolean;
// }

// export interface ChatMessage {
//   role: "user" | "assistant" | "system" | "function";
//   content: ChatMessageContent;
//   image?: string;
// }

export type ChatMessage =
  | UserChatMessage
  | AssistantChatMessage
  | SystemChatMessage;

export interface UserChatMessage {
  role: "user";
  content: TextContent | FunctionCallResponseContent;
  image?: string;
}

export interface AssistantChatMessage {
  role: "assistant";
  content: ChatMessageContent;
  image?: string;
}

export interface SystemChatMessage {
  role: "system";
  content: TextContent;
}

export interface EmittedChatMessage {
  id: string;
  message: ChatMessage[];
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

export interface Text {
  text: string;
  _meta?: Record<string, any>;
}

export interface TextContent {
  type: "text";
  content: Text;
}

export interface FunctionCallRequest {
  id?: string;
  name: string;
  args?: any;
  _meta?: Record<string, any>;
}

export interface FunctionCallRequestContent {
  type: "functionCallRequest";
  content: FunctionCallRequest;
}

export interface FunctionCallResponse {
  id?: string;
  name: string;
  response: any;
  _meta?: Record<string, any>;
}

export interface FunctionCallResponseContent {
  type: "functionCallResponse";
  content: FunctionCallResponse;
}

export type ChatMessageContent =
  | TextContent
  | FunctionCallRequestContent
  | FunctionCallResponseContent;

export type FunctionCallStatus = "awaiting" | "success" | "failed";

export interface AwaitingFunctionCall {
  id: string;
  functionCall: FunctionCallRequest;
  status: FunctionCallStatus;
  responseId: string;

  serverName: string;
  functionName: string;
}

export interface EmittedMCPResponse {
  requestId: string;
  responseId: string;

  response: any;
}
