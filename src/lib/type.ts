export interface ChatMessage {
	id: string,
	role: "user" | "assistant" | "system",
	content: string,
	image?: string
}

export interface EmittedChatMessage {
	id: string,
	message: ChatMessage,
	done: boolean
}
