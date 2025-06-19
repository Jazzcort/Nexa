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

  let {
    chatMessage,
    id,
    index,
    handleInputBoxSelection,
  }: {
    chatMessage: ChatMessageWithId;
    id: string;
    index: number;
    handleInputBoxSelection: (index: number) => void;
  } = $props();

  // Editor states
  let editor = $state<Editor>();
  let showToolBar = $state(true);

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

  // const onUpdate = (props: { editor: Editor, transaction: Transaction }) => {
  //   editor.f
  // }

  onMount(() => {
    editor?.on("focus", () => {
      handleInputBoxSelection(index);
    });
  });

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
    <Edra
      class="h-fit overflow-auto"
      bind:editor
      content={chatMessage.content}
      editable={chatMessage.role === "user" ? true : false}
    />
  </div>
</div>
