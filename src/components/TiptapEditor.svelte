<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";

  let props = $props();

  let element;
  let editor: Editor | null;

  onMount(() => {
    editor = new Editor({
      element: element,
      extensions: [StarterKit],
      content: props.content,
      onTransaction: () => {
        editor = editor;
      },
    });
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });
</script>

<!-- {#if editor} -->
<!--   <button -->
<!--     on:click={() => editor!.chain().focus().toggleHeading({ level: 1 }).run()} -->
<!--     class:active={editor.isActive("heading", { level: 1 })} -->
<!--   > -->
<!--     H1 -->
<!--   </button> -->
<!--   <button -->
<!--     on:click={() => editor!.chain().focus().toggleHeading({ level: 2 }).run()} -->
<!--     class:active={editor.isActive("heading", { level: 2 })} -->
<!--   > -->
<!--     H2 -->
<!--   </button> -->
<!--   <button -->
<!--     on:click={() => editor!.chain().focus().setParagraph().run()} -->
<!--     class:active={editor.isActive("paragraph")} -->
<!--   > -->
<!--     P -->
<!--   </button> -->
<!-- {/if} -->

<div id={props.id} bind:this={element}></div>

<style>
  button.active {
    background: black;
    color: white;
  }
</style>
