<script lang="ts">
	import * as Collapsible from "$lib/components/ui/collapsible/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
	import { Button } from "$lib/components/ui/button/index";
	import {
		type AwaitingFunctionCall,
		type FunctionCallStatus,
	} from "$types";
	import type { HTMLAttributes } from "svelte/elements";
	import { invoke } from "@tauri-apps/api/core";

	type CustomProps = {
		class?: string;
		awaitingFunctionCall: AwaitingFunctionCall | undefined;
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
		console.log("called!");
		invoke("call_tool", {
			serverName: awaitingFunctionCall?.serverName,
			functionName: awaitingFunctionCall?.functionName,
			requestId: awaitingFunctionCall?.id,
			responseId: awaitingFunctionCall?.responseId,
			arguments: awaitingFunctionCall?.functionCall.args,
		});
	};
</script>

{#if awaitingFunctionCall}
	<Collapsible.Root class={className}>
		<div class="flex items-center justify-between space-x-4 px-4">
			<h4 class="text-sm font-semibold">
				{awaitingFunctionCall.functionName}
			</h4>
			<Collapsible.Trigger
				class={buttonVariants({
					variant: "ghost",
					size: "sm",
					class: "w-9 p-0",
				})}
			>
				<ChevronsUpDownIcon />
				<span class="sr-only">Toggle</span>
			</Collapsible.Trigger>
			<Button onclick={triggerToolCall}>run</Button>
		</div>
		<Collapsible.Content class="space-y-2">
			{#if functionCallResponse}
				<div>
					<p class="text-sm font-bold">
						RESPONSE
					</p>
					{JSON.stringify(functionCallResponse)}
				</div>
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
