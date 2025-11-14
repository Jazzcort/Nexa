<script lang="ts">
	import { ScrollArea } from "$lib/components/ui/scroll-area/index";
	import Item from "./components/Item.svelte";
	import Settings from "@lucide/svelte/icons/settings";
</script>

<main class="config-container my-2 mx-0">
	<ScrollArea class="px-8 py-2 border max-w-xs grow ml-16">
		<Item content="General" icon={Settings} />
		<Item content="General" icon={Settings} />
		<Item content="General" icon={Settings} />
		<Item content="General" icon={Settings} />
		<Item content="General" icon={Settings} />
	</ScrollArea>
	<ScrollArea class="grow-2 max-w-3xl mr-16 border">
		<h1 class="text-red-500">Config</h1>
		<a href="/">Home</a>
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
