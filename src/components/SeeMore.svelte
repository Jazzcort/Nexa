<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { cn } from "$lib/utils"; // Shadcn utility for merging classes

	let {
		header,
		text = "",
		limit = 150,
		class: className,
	}: {
		header?: string;
		text: string;
		limit: number;
		class?: string;
	} = $props();

	let expanded = $state(false);

	const isTooLong = text.length > limit;
	const displayText = $derived(
		expanded || !isTooLong
			? text
			: text.slice(0, limit).trim() + "...",
	);
</script>

<div class={cn("text-sm", className)}>
	{#if header}
		<p class="inline font-mono text-foreground font-bold">
			{header}
		</p>
	{/if}
	<p class="inline break-all text-muted-foreground">
		{displayText}
	</p>

	{#if isTooLong}
		<Button
			variant="link"
			size="sm"
			class="h-auto p-0 ml-1 font-semibold text-primary"
			onclick={() => (expanded = !expanded)}
		>
			{expanded ? "See less" : "See more"}
		</Button>
	{/if}
</div>
