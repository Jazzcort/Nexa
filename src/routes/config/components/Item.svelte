<script lang="ts">
	import * as Item from "$lib/components/ui/item/index.js";
	import BadgeCheckIcon from "@lucide/svelte/icons/badge-check";
	import type { Component } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type CustomProps = {
		class?: string;
		title: string;
		icon: Component;
	};

	type RootHtmlProps = HTMLAttributes<HTMLDivElement>;

	type Props = CustomProps & RootHtmlProps;

	let {
		class: className,
		title,
		icon: Icon,
		...restProps
	}: Props = $props();
</script>

<Item.Root class={className} {...restProps}>
	<Item.Media>
		<Icon class="size-5" />
	</Item.Media>
	<Item.Title>{title}</Item.Title>
</Item.Root>
