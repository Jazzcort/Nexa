<script lang="ts">
	import { ScrollArea as ScrollAreaPrimitive } from "bits-ui";
	import { Scrollbar } from "./index.js";
	import { cn } from "$lib/utils.js";

	type $$Props = ScrollAreaPrimitive.Props & {
		orientation?: "vertical" | "horizontal" | "both";
		scrollbarXClasses?: string;
		scrollbarYClasses?: string;
		hideScrollBar?: boolean;
	};

	let className: $$Props["class"] = undefined;
	export { className as class };
	export let orientation = "vertical";
	export let scrollbarXClasses: string = "";
	export let scrollbarYClasses: string = "";
	export let hideScrollBar: boolean = false;
</script>

<ScrollAreaPrimitive.Root
	{...$$restProps}
	class={cn("relative overflow-hidden", className)}
>
	<ScrollAreaPrimitive.Viewport class="h-full w-full rounded-[inherit]">
		<ScrollAreaPrimitive.Content>
			<slot />
		</ScrollAreaPrimitive.Content>
	</ScrollAreaPrimitive.Viewport>
	{#if orientation === "vertical" || orientation === "both"}
		<Scrollbar
			orientation="vertical"
			{hideScrollBar}
			class={scrollbarYClasses}
		/>
	{/if}
	{#if orientation === "horizontal" || orientation === "both"}
		<Scrollbar
			orientation="horizontal"
			{hideScrollBar}
			class={scrollbarXClasses}
		/>
	{/if}
	<ScrollAreaPrimitive.Corner />
</ScrollAreaPrimitive.Root>
