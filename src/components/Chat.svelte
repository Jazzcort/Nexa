<script lang="ts">
  import type { EmittedChatMessage, ChatMessage } from "$lib/type";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { Textarea } from "$lib/components/ui/textarea/index";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import CustomButton from "./CustomButton.svelte";
  import MessageBox from "./MessageBox.svelte";
  import { onMount, tick } from "svelte";
  import { v4 as uuidv4 } from "uuid";

  let scrollDown: HTMLElement | null;
  let chatSendBtn: HTMLElement | null;

  onMount(() => {
    chatSendBtn = document.getElementById("chat-send-btn");

    window.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        testAsync();
      }
    });
  });

  listen<EmittedChatMessage>("llm-response", (event) => {
    if (chatHistory.length <= 0) {
      return;
    }

    const idx = chatHistory.length - 1;
    if (chatHistory[idx].id !== event.payload.id) {
      return;
    }

    if (!event.payload.done) {
      chatHistory[idx].content += event.payload.message.content;

      if (scrollDown) {
        scrollDown.scrollIntoView({ behavior: "instant", block: "end" });
      }
    } else {
      disableSendButton = false;
    }
  });

  let disableSendButton = $state(false);
  let userMessage: ChatMessage = $state({
    id: uuidv4(),
    role: "user",
    content: "",
  });
  let chatHistory: ChatMessage[] = $state([]);

  const testAsync = async () => {
    chatHistory.push({ ...userMessage });
    chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: "",
    });
    disableSendButton = true;
    invoke("test_async", { messages: chatHistory });

    await tick();
    scrollDown = document.getElementById(
      `message-box-${chatHistory.length - 1}`,
    );

    if (scrollDown) {
      scrollDown.scrollIntoView({ behavior: "instant", block: "end" });
    }
    setTimeout(() => {
      userMessage.content = "";
      userMessage.id = uuidv4();
    }, 80);
  };
</script>

<main class="flex flex-col h-full w-full">
  <ScrollArea
    id="chat-history"
    class="h-full border border-black m-2 rounded-xl"
    hideScrollBar
  >
    {#each chatHistory as msg, i}
      <MessageBox id={`message-box-${i}`} content={msg.content} />
    {/each}
    <span id="scroll-down" class="opacity-0">hi</span>
  </ScrollArea>
  <div class="border h-px w-full"></div>
  <div class="m-2 h-fit">
    <Textarea
      placeholder="Type your message here."
      bind:value={userMessage.content}
    />
    <div class="flex flex-row-reverse w-full py-2">
      <CustomButton
        id="chat-send-btn"
        class=""
        buttonText="Send"
        onclick={testAsync}
        disable={disableSendButton}
      />
    </div>
  </div>
</main>
