import { Store } from "@tauri-apps/plugin-store";
import type { ChatMessageWithId } from "$types";

class ChatHistoryStore {
  isReady = $state(false);
  chatHistory = $state<ChatMessageWithId[]>([]);

  #store: Store | null = null;
  #FILENAME = "chat-history.json";

  constructor() {
    this.init();
  }

  async init() {
    try {
      this.#store = await Store.load(this.#FILENAME);
      const savedChatHistory =
        await this.#store.get<ChatMessageWithId[]>("chatHistory");

      if (savedChatHistory) {
        this.chatHistory = savedChatHistory;
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

  async sync(chatHistory: ChatMessageWithId[]) {
    this.chatHistory = chatHistory;
    await this.#save("chatHistory", this.chatHistory);
    console.log("save!!");
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
