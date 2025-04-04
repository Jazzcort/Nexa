<script lang="ts">
  import type { EmittedChatMessage, ChatMessage } from "$lib/type";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import CustomButton from "./CustomButton.svelte";
  // listen<string>("testing", (event) => {
  //   responseText += event.payload;
  // });

  listen<EmittedChatMessage>("llm-response", (event) => {
    if (!event.payload.done) {
      responseText += event.payload.message.contexnt;
    }
  });

  let responseText = $state("");
  let inputText = $state("");
  let chatHistory: ChatMessage[] = $state([]);

  const handleClick = () => {
    invoke("emit_events");
  };

  const testAsync = () => {
    invoke("test_async", { messages: chatHistory });
  };
</script>

<main class="flex flex-col h-full w-full">
  <div class="h-full border m-2 rounded-xl">
    <Textarea readonly class="h-full" value={responseText} />
    <!-- <p>{inputText}</p> -->
  </div>
  <div class="border h-px w-full"></div>
  <div class="m-2">
    <Textarea placeholder="Type your message here." bind:value={inputText} />
    <div class="flex flex-row-reverse w-full py-2">
      <CustomButton class="" buttonText="Send" onclick={handleClick} />
      <button onclick={testAsync}>test!</button>
    </div>
  </div>
</main>
