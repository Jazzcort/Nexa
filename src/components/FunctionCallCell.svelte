<script lang="ts">
	import * as Collapsible from "$lib/components/ui/collapsible/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
	import { Button } from "$lib/components/ui/button/index";
	import { type FunctionCallInfo, type FunctionCallStatus } from "$types";
	import type { HTMLAttributes } from "svelte/elements";
	import { invoke } from "@tauri-apps/api/core";
	import { Spinner } from "$lib/components/ui/spinner/index.js";
	import Dot from "@lucide/svelte/icons/dot";
	import Check from "@lucide/svelte/icons/check";
	import Cross from "@lucide/svelte/icons/cross";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index";
	import SeeMore from "./SeeMore.svelte";

	type CustomProps = {
		class?: string;
		awaitingFunctionCall: FunctionCallInfo | undefined;
		functionCallResponse?: any;
	};
	type RootHtmlProps = HTMLAttributes<HTMLDivElement>;
	type Props = CustomProps & RootHtmlProps;
	let {
		class: className,
		awaitingFunctionCall,
		functionCallResponse,
		...restProps
	}: Props = $props();

	const triggerToolCall = () => {
		if (
			awaitingFunctionCall &&
			awaitingFunctionCall.status === "initialized"
		) {
			awaitingFunctionCall.status = "awaiting";
			invoke("call_tool", {
				serverName: awaitingFunctionCall.serverName,
				functionName: awaitingFunctionCall.functionName,
				requestId: awaitingFunctionCall.id,
				responseId: awaitingFunctionCall.responseId,
				arguments: awaitingFunctionCall.functionCall
					.args,
			});
		}
	};
</script>

{#if awaitingFunctionCall}
	<!-- <div> -->
	<!-- 	{`id: ${awaitingFunctionCall.id}\nstatus: ${awaitingFunctionCall.status}`} -->
	<!-- </div> -->
	<!-- <div> -->
	<!-- 	{`response: ${JSON.stringify(functionCallResponse)}\nresponseId: ${awaitingFunctionCall.responseId}`} -->
	<!-- </div> -->
	<Collapsible.Root class={className}>
		<div class="flex items-center justify-between px-4 m-0">
			<div class="flex items-center">
				{#if awaitingFunctionCall.status === "awaiting"}
					<Spinner class="mr-2 size-2" />
				{:else if awaitingFunctionCall.status === "success"}
					<Check class="mr-2 size-2" />
				{:else if awaitingFunctionCall.status === "failed"}
					<Cross class="mr-2 size-2" />
				{:else}
					<Dot class="mr-2 size-2" />
				{/if}
				<h4 class="text-sm font-semibold truncate">
					{awaitingFunctionCall.functionName}
				</h4>
			</div>
			<div class="flex items-center">
				<Collapsible.Trigger
					class={buttonVariants({
						variant: "ghost",
						size: "sm",
						class: "w-9 p-0 mr-2",
					})}
				>
					<ChevronsUpDownIcon />
					<span class="sr-only">Toggle</span>
				</Collapsible.Trigger>
				<Button onclick={triggerToolCall}>run</Button>
			</div>
		</div>
		<Collapsible.Content class="space-y-2 py-4">
			{#if Object.keys(functionCallResponse).length !== 0}
				<p class="text-sm font-bold">RESPONSE</p>
				<SeeMore
					class=""
					text={JSON.stringify(
						functionCallResponse,
					)}
					limit={250}
				/>
				<!-- <ScrollArea -->
				<!-- 	class="w-full h-[250px] overflow-hidden" -->
				<!-- 	>{JSON.stringify( -->
				<!-- 		functionCallResponse, -->
				<!-- 	)}</ScrollArea -->
				<!-- > -->
			{/if}
			{#each Object.entries(awaitingFunctionCall.functionCall.args) as arg}
				<div
					class="rounded-md border px-4 py-3 font-mono text-sm"
				>
					<strong>{arg[0]}</strong>{`: ${arg[1]}`}
				</div>
			{/each}
		</Collapsible.Content>
	</Collapsible.Root>
{/if}
