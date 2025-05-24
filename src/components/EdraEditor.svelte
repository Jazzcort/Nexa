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

  let { inputContent, id } = $props();

  // Editor states
  let content = $state<Content>();
  let editor = $state<Editor>();
  let showToolBar = $state(true);

  function onUpdate(props: { editor: Editor; transaction: Transaction }) {
    content = props.editor.getJSON();
  }

  onMount(() => {
    editor?.commands.setContent(inputContent);
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
    <Edra class="h-fit overflow-auto" bind:editor {content} {onUpdate} />
  </div>
</div>
