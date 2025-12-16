import { Store } from "@tauri-apps/plugin-store";
import type {
  ChatMessageWithId,
  FunctionCallInfo,
  FunctionCallRequest,
  FunctionCallStatus,
} from "$types";

export class ReactiveFunctionCallInfo implements FunctionCallInfo {
  id: string;
  functionCall: FunctionCallRequest;
  status: FunctionCallStatus;
  responseId: string;
  serverName: string;
  functionName: string;

  constructor(info: FunctionCallInfo) {
    this.id = info.id;
    this.functionCall = info.functionCall;
    this.status = $state(info.status);
    this.responseId = info.responseId;

    this.serverName = info.serverName;
    this.functionName = info.functionName;
  }

  toJSON(): FunctionCallInfo {
    return {
      id: this.id,
      status: this.status,
      functionCall: this.functionCall,
      responseId: this.responseId,
      serverName: this.serverName,
      functionName: this.functionName,
    };
  }
}

class ChatHistoryStore {
  isReady = $state(false);
  chatHistory = $state<ChatMessageWithId[]>([]);
  functionCallInfo = $state<Map<string, ReactiveFunctionCallInfo>>(new Map());

  #store: Store | null = null;
  #FILENAME = "chat-history.json";

  constructor() {
    this.init();
  }

  async init() {
    try {
      this.#store = await Store.load(this.#FILENAME);
      // Loading chat history
      const savedChatHistory =
        await this.#store.get<ChatMessageWithId[]>("chatHistory");
      if (savedChatHistory) {
        this.chatHistory = savedChatHistory;
      }

      // Loading function call info
      const savedFunctionCallInfo =
        await this.#store.get<Record<string, any>>("functionCallInfo");
      if (savedFunctionCallInfo) {
        const validatedFunctionCallInfo: Map<string, ReactiveFunctionCallInfo> =
          new Map();
        const savedFunctionCallInfoMap = new Map(
          Object.entries(savedFunctionCallInfo),
        );
        this.chatHistory.forEach((msg) => {
          if (
            msg.content.type === "functionCallRequest" &&
            savedFunctionCallInfoMap.has(msg.id)
          ) {
            validatedFunctionCallInfo.set(
              msg.id,
              new ReactiveFunctionCallInfo(
                savedFunctionCallInfoMap.get(msg.id)!,
              ),
            );
          }
        });

        this.functionCallInfo = validatedFunctionCallInfo;
      }

      this.isReady = true;
    } catch (e) {
      console.error("Failed to load settings", e);
    }
  }

  async addMessage(msg: ChatMessageWithId) {
    this.chatHistory.push(msg);
    await this.#save("chatHistory", this.chatHistory);
  }

  async syncChatHistory(chatHistory: ChatMessageWithId[]) {
    this.chatHistory = chatHistory;
    await this.#save("chatHistory", this.chatHistory);
    console.log("chat history save!!");
  }

  async addFunctionCallInfo(functionCallInfo: ReactiveFunctionCallInfo[]) {
    if (functionCallInfo.length === 0) {
      return;
    }

    functionCallInfo.forEach((info) =>
      this.functionCallInfo.set(info.id, new ReactiveFunctionCallInfo(info)),
    );
    await this.#save(
      "functionCallInfo",
      Object.fromEntries(
        [...this.functionCallInfo].map(([key, fcInfo]) => [
          key,
          fcInfo.toJSON(),
        ]),
      ),
    );
    console.log("function call info added!");
  }

  async saveChatHistory() {
    await this.#save("chatHistory", this.chatHistory);
    console.log("chat history save!!");
  }

  async saveFunctionCallInfo() {
    console.log(Object.fromEntries(this.functionCallInfo), "entries!!!!");
    await this.#save(
      "functionCallInfo",
      Object.fromEntries(
        [...this.functionCallInfo].map(([key, fcInfo]) => [
          key,
          fcInfo.toJSON(),
        ]),
      ),
    );
    console.log("function call info save!!");
  }

  async #save(key: string, value: any) {
    if (!this.#store) {
      return;
    }

    try {
      await this.#store.set(key, value);
      await this.#store.save();
    } catch (e) {
      console.error(`Failed to save ${key}:`, e);
    }
  }
}

export const chatHistoryStore = new ChatHistoryStore();
