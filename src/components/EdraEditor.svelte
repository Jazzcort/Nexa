<script lang="ts">
  import type { Content, Editor } from "@tiptap/core";
  import type { Transaction } from "@tiptap/pm/state";
  import {
    Edra,
    EdraToolbar,
    EdraBubbleMenu,
  } from "$lib/components/edra/shadcn/index.js";
  import DragHandle from "$lib/components/edra/drag-handle.svelte";
  import { onMount, onDestroy } from "svelte";
  import type { ChatMessageWithId } from "$lib/type";

  // let { inputContent, id }: { inputContent: string; id: string } = $props();
  let { chatMessage, id }: { chatMessage: ChatMessageWithId; id: string } =
    $props();

  // Editor states
  // let content = $state<Content>();
  let editor = $state<Editor>();
  let showToolBar = $state(true);

  // function onUpdate(props: { editor: Editor; transaction: Transaction }) {
  //   content = props.editor.getJSON();
  // }

  // onMount(() => {
  //   editor?.commands.setContent(inputContent);
  // });

  // $effect(() => {
  //   updateContent(inputContent);
  // });

  const intervalId = setInterval(() => {
    updateContent(chatMessage.content);
  }, 50);

  $effect(() => {
    if (chatMessage.done) {
      clearInterval(intervalId);
    }
  });

  const updateContent = (content: string) => {
    editor?.commands.setContent(content);
  };

  onDestroy(() => {
    editor?.destroy();
  });
</script>

<div {id} class="m-2">
  {#if editor && showToolBar}
    <!-- <div class="overflow-auto rounded-t border-x border-t p-1"> -->
    <!--   <EdraToolbar {editor} /> -->
    <!-- </div> -->
    <!-- <EdraBubbleMenu {editor} /> -->
    <!-- <DragHandle {editor} /> -->
  {/if}
  <div class="w-full rounded border">
    <!-- <Edra class="h-fit overflow-auto" bind:editor {content} {onUpdate} /> -->
    <Edra
      class="h-fit overflow-auto"
      bind:editor
      content={chatMessage.content}
      editable={chatMessage.role === "user" ? true : false}
    />
  </div>
</div>
