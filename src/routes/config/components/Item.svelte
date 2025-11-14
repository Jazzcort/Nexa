<script lang="ts">
	import * as Item from "$lib/components/ui/item/index.js";
	import BadgeCheckIcon from "@lucide/svelte/icons/badge-check";
	import type { Component } from "svelte";
	let {
		class: className,
		content,
		icon: Icon,
	}: {
		class?: string;
		content: string;
		icon: Component;
	} = $props();
</script>

<Item.Root class={className}>
	{#snippet child({ props })}
		<a href="#/" {...props}>
			<Item.Media>
				<Icon class="size-5" />
			</Item.Media>
			<Item.Content>{content}</Item.Content>
		</a>
	{/snippet}
</Item.Root>
