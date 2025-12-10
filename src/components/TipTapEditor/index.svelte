<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { Editor } from "@tiptap/core";
	import StarterKit from "@tiptap/starter-kit";
	import { all, createLowlight } from "lowlight";
	import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
	import TurndownService from "turndown";
	import { gfm } from "turndown-plugin-gfm";
	import Code from "@tiptap/extension-code";
	import Highlight from "@tiptap/extension-highlight";
	import type { ChatMessageWithId, Text } from "$types";
	import js from "highlight.js/lib/languages/javascript";
	import rust from "highlight.js/lib/languages/rust";
	import python from "highlight.js/lib/languages/python";
	import typescript from "highlight.js/lib/languages/typescript";
	import java from "highlight.js/lib/languages/java";
	import cpp from "highlight.js/lib/languages/cpp";
	import json from "highlight.js/lib/languages/json";
	import bash from "highlight.js/lib/languages/bash";
	import { SvelteNodeViewRenderer } from "svelte-tiptap";
	import {
		FontSize,
		TextStyle,
		Color,
	} from "@tiptap/extension-text-style";
	import { Markdown } from "tiptap-markdown";
	import { ColorHighlighter } from "./extensions/ColorHighlighter";
	import { CodeTagMarkdown } from "./extensions/CodeTagMarkdown";
	import { EnterDisabler } from "./extensions/EnterDisabler";
	import Bold from "@tiptap/extension-bold";
	import {
		parseMarkdownToTipTap,
		hasIncompleteCodeBlock,
	} from "./markdownParser";
	import "./editor.css";
	import CodeBlock from "./CodeBlock.svelte";
	import {
		AssistantEditorExtension,
		UserEditorExtension,
	} from "./extensionsBundles";

	let element: any;
	let editor: Editor;

	const lowlight = createLowlight(all);
	const turndownService = new TurndownService({
		headingStyle: "atx",
	});
	turndownService.use(gfm);

	lowlight.register("rust", rust);
	lowlight.register("js", js);
	lowlight.register("javascript", js);
	lowlight.register("python", python);
	lowlight.register("py", python);
	lowlight.register("typescript", typescript);
	lowlight.register("ts", typescript);
	lowlight.register("java", java);
	lowlight.register("cpp", cpp);
	lowlight.register("c++", cpp);
	lowlight.register("json", json);
	lowlight.register("bash", bash);
	lowlight.register("sh", bash);

	let {
		content,
		id,
		index,
		handleInputBoxSelection,
		triggerStreamChat,
	}: {
		content: ChatMessageWithId;
		id: string;
		index: number;
		handleInputBoxSelection: (index: number) => void;
		triggerStreamChat: (
			index: number,
			modifiedContent: ChatMessageWithId,
		) => void;
	} = $props();

	const handleKeyDown = (e: KeyboardEvent) => {
		console.log(e);
		switch (e.key) {
			case "Enter":
				if (e.shiftKey) {
					// Add a new line, do nothing here
				} else {
					const text = editor.getText({
						blockSeparator: "\n",
					});

					const modifiedContent = {
						...content,
					};

					(
						modifiedContent.content
							.content as Text
					).text = text;

					console.log("trigger!!");

					triggerStreamChat(
						index,
						modifiedContent,
					);
				}

			// Testing purpose
			// const text = editor.getText();
			// console.log(text);
			// const jsonObject = editor.getJSON();
			// console.log(jsonObject);
		}
	};

	const updateEditorContent = (newContent: string) => {
		const parsedContent =
			content.role === "assistant"
				? parseMarkdownToTipTap(
						(
							content.content
								.content as Text
						).text,
					)
				: newContent
						.split("\n")
						.map((line) => `<p>${line}</p>`)
						.join("");
		editor.commands.setContent(parsedContent);
	};

	onMount(() => {
		editor = new Editor({
			element: element,
			extensions:
				content.role === "assistant"
					? AssistantEditorExtension
					: UserEditorExtension,
			content: "",
			onTransaction: (prop) => {
				// force re-render so `editor.isActive` works as expected
				editor = editor;
			},
			onFocus: () => {
				handleInputBoxSelection(index);
			},
			editable: content.role === "user",
		});

		// This is important for the initial message in the user's message box
		updateEditorContent((content.content.content as Text).text);

		element.addEventListener("keydown", handleKeyDown);
		return () => {
			element.removeEventListener("keydown", handleKeyDown);
		};
	});

	$effect(() => {
		let interval: number | undefined = undefined;
		if (content.done) {
			// Delay interval clear so we can get the last streaming message
			updateEditorContent(
				(content.content.content as Text).text,
			);
			clearInterval(interval);
		} else {
			interval = setInterval(() => {
				updateEditorContent(
					(content.content.content as Text).text,
				);
			}, 100);
		}

		return () => {
			clearInterval(interval);
		};
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
		}
	});
</script>

<div class="">
	<div
		class="border border-black rounded p-2 m-2 px-4"
		{id}
		bind:this={element}
	></div>
	<!-- <p>{content.content}</p> -->
</div>

<style>
	button.active {
		background: black;
		color: white;
	}
</style>
