<script lang="ts">
	import { Input } from "$lib/components/ui/input/index.js";
	import * as Item from "$lib/components/ui/item/index.js";
	import Eye from "@lucide/svelte/icons/eye";
	import EyeOff from "@lucide/svelte/icons/eye-off";
	import { invoke } from "@tauri-apps/api/core";
	import Button from "$lib/components/ui/button/button.svelte";
	import { onMount } from "svelte";
	import type { GetItemResponse } from "$types";
	import { GEMINI_KEY_NAME } from "$lib/constants";

	// Display control
	let displayGeminiAPIKey = $state(false);

	// Inputs for API keys
	let geminiAPIKeyInput = $state("");

	const saveAPIKey = async (key: string, value: string) => {
		await invoke("plugin:secure-storage|set_item", {
			payload: {
				prefixedKey: key,
				data: value,
			},
		});
	};

	const getAPIKey = async (key: string) => {
		const res: GetItemResponse = await invoke(
			"plugin:secure-storage|get_item",
			{
				payload: {
					prefixedKey: key,
				},
			},
		);

		return res.data ? res.data : "";
	};

	const loadKeys = async () => {
		// Gemini
		geminiAPIKeyInput = await getAPIKey(GEMINI_KEY_NAME);
	};

	onMount(() => {
		loadKeys();
	});
</script>

<div class="flex flex-col justify-center items-center">
	<h1>API Keys</h1>
	<h3>In Progress 🚧</h3>

	<Item.Root variant="outline" class="w-[90%]">
		<Item.Content>
			<Item.Title>Gemini API Key</Item.Title>
			<div class="flex items-center">
				<Input
					autocomplete="off"
					autocorrect="off"
					autocapitalize="none"
					spellcheck="false"
					class="my-2"
					type={displayGeminiAPIKey
						? "text"
						: "password"}
					placeholder="Enter your Gemini API Key"
					bind:value={geminiAPIKeyInput}
					onblur={() =>
						saveAPIKey(
							GEMINI_KEY_NAME,
							geminiAPIKeyInput,
						)}
				/>

				<div class="mx-2">
					{#if displayGeminiAPIKey}
						<EyeOff
							class="hover:stroke-violet-300 transition"
							onclick={() =>
								(displayGeminiAPIKey = false)}
						/>
					{:else}
						<Eye
							class="hover:stroke-violet-300 transition"
							onclick={() =>
								(displayGeminiAPIKey = true)}
						/>
					{/if}
				</div>
			</div>
		</Item.Content>
	</Item.Root>
</div>
