export interface ChatMessage {
	role: "user" | "assistant" | "system",
	contexnt: string,
	image?: string
}

export interface Haha {
	content: string
}

export interface EmittedChatMessage {
	message: ChatMessage,
	done: boolean
}
