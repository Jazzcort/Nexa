
export interface ChatMessage {
	role: "user" | "assistant" | "system",
	content: string,
	image?: string
}

export interface EmittedChatMessage {
	message: ChatMessage,
	done: boolean
}
