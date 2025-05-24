<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import CodeBlock from "@tiptap/extension-code-block";
  import { createLowlight, common } from "lowlight";
  import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";

  const lowlight = createLowlight(common);

  let props = $props();

  $effect(() => {
    // editor?.commands.setContent(props.content, true);
    editor?.destroy();
    //
    editor = new Editor({
      element: element,
      extensions: [
        StarterKit,
        CodeBlockLowlight.configure({
          lowlight,
          languageClassPrefix: "language-",
          defaultLanguage: "python",
        }),
      ],
      content: `
        <p>
          That’s a boring paragraph followed by a fenced code block:
        </p>
        <pre><code class="language-python">for (var i=1; i <= 20; i++)
{
  if (i % 15 == 0)
    console.log("FizzBuzz");
  else if (i % 3 == 0)
    console.log("Fizz");
  else if (i % 5 == 0)
    console.log("Buzz");
  else
    console.log(i);
}</code></pre>
        <p>
          Press Command/Ctrl + Enter to leave the fenced code block and continue typing in boring paragraphs.
        </p>
      `,
      injectCSS: true,
      onTransaction: ({ editor }) => {
        editor = editor;
      },
    });
  });

  let element: any;
  let editor: Editor | null;

  // onMount(() => {
  //   editor = new Editor({
  //     element: element,
  //     extensions: [StarterKit],
  //     content: props.content,
  //     onTransaction: () => {
  //       editor = editor;
  //     },
  //     onUpdate: () => {
  //
  //     }
  //   });
  // });

  onDestroy(() => {
    editor?.destroy();
  });
</script>

<div class="m-1" id={props.id} bind:this={element}></div>
<button class="haha" onclick={() => editor?.commands.toggleCodeBlock()}
  >update</button
>

<style>
  .haha {
    background-color: red;
  }
  .tiptap {
    :first-child {
      margin-top: 0;
    }

    pre {
      background: var(--black);
      border-radius: 0.5rem;
      color: var(--white);
      font-family: "JetBrainsMono", monospace;
      margin: 1.5rem 0;
      padding: 0.75rem 1rem;

      code {
        background: none;
        color: inherit;
        font-size: 0.8rem;
        padding: 0;
      }

      /* Code styling */
      .hljs-comment,
      .hljs-quote {
        color: #616161;
      }

      .hljs-variable,
      .hljs-template-variable,
      .hljs-attribute,
      .hljs-tag,
      .hljs-regexp,
      .hljs-link,
      .hljs-name,
      .hljs-selector-id,
      .hljs-selector-class {
        color: #f98181;
      }

      .hljs-number,
      .hljs-meta,
      .hljs-built_in,
      .hljs-builtin-name,
      .hljs-literal,
      .hljs-type,
      .hljs-params {
        color: #fbbc88;
      }

      .hljs-string,
      .hljs-symbol,
      .hljs-bullet {
        color: #b9f18d;
      }

      .hljs-title,
      .hljs-section {
        color: #faf594;
      }

      .hljs-keyword,
      .hljs-selector-tag {
        color: #70cff8;
      }

      .hljs-emphasis {
        font-style: italic;
      }

      .hljs-strong {
        font-weight: 700;
      }
    }
  }
</style>
