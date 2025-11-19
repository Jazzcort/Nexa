<script lang="ts">
	import { ScrollArea } from "$lib/components/ui/scroll-area/index";
	import Item from "./components/Item.svelte";
	import Settings from "@lucide/svelte/icons/settings";
	import KeyRound from "@lucide/svelte/icons/key-round";
	import type { Component } from "svelte";
	import type { ConfigSection } from "$types";
	import Sections from "./Sections/index.svelte";

	interface SelectionOption {
		section: ConfigSection;
		title: string;
		icon: Component;
	}

	let curSelection = $state<ConfigSection>("general");

	const handleSelection = (selection: ConfigSection) => {
		curSelection = selection;
	};

	const selectionOptions: SelectionOption[] = [
		{
			section: "general",
			title: "General",
			icon: Settings,
		},
		{
			section: "apiKeys",
			title: "API Keys",
			icon: KeyRound,
		},
	];
</script>

<main class="config-container my-2 mx-0">
	<ScrollArea class="px-8 py-2 border max-w-2xs grow ml-16">
		{#each selectionOptions as option}
			<Item
				class={`my-2 ${
					curSelection === option.section
						? "bg-violet-200"
						: "hover:bg-violet-50"
				}`}
				title={option.title}
				icon={option.icon}
				onclick={() => handleSelection(option.section)}
			></Item>
		{/each}
		<h1 class="text-red-500">Config</h1>
		<a href="/">Home</a>
	</ScrollArea>
	<ScrollArea class="grow-2 max-w-3xl mr-16">
		<Sections section={curSelection} />
	</ScrollArea>
</main>

<style>
	/* Make your app container grow to fill the space */
	.config-container {
		flex-grow: 1;
		display: flex;
		flex-direction: row;
		overflow: hidden;
		justify-content: center;
	}
</style>
