<script lang="ts">
  import type {
    EmittedChatMessage,
    ChatMessageWithId,
    ChatMessage,
  } from "$lib/type";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { Textarea } from "$lib/components/ui/textarea/index";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import CustomButton from "./CustomButton.svelte";
  import { onMount, tick } from "svelte";
  import { v4 as uuidv4 } from "uuid";
  import EdraEditor from "./EdraEditor.svelte";
  import ModelSelectionDropdownMenu from "./ModelSelectionDropdownMenu.svelte";
  import { modelState } from "../states/modelState.svelte";

  let scrollDown: HTMLElement | null;
  let chatSendBtn: HTMLElement | null;
  let userInputBox: HTMLElement | null;
  let disableSendButton = $state(false);
  let userMessage: ChatMessage = $state({
    role: "user",
    content: "",
  });
  let chatHistory: ChatMessageWithId[] = $state([]);
  let currentInputBoxIndex = $state(0);

  const handleModelSelection = (index: number) => {
    modelState.index = index;
  };

  const handleInputBoxSelection = (index: number) => {
    currentInputBoxIndex = index;
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "Enter":
        if (currentInputBoxIndex === chatHistory.length) {
          streamChat();
        } else {
          // TODO: handle message chopping
        }
        break;
    }
  };

  const streamChat = async () => {
    chatHistory.push({ ...userMessage, id: uuidv4(), done: true });
    chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: "",
      done: false,
    });
    disableSendButton = true;
    invoke("stream_chat", {
      messages: chatHistory,
      model: modelState.models[modelState.index],
    });

    await tick();
    scrollDown = document.getElementById(
      `message-box-${chatHistory.length - 1}`,
    );

    if (scrollDown) {
      scrollDown.scrollIntoView({ behavior: "instant", block: "end" });
    }
    setTimeout(() => {
      userMessage.content = "";
      userInputBox?.focus();
    }, 80);
  };

  onMount(() => {
    chatSendBtn = document.getElementById("chat-send-btn");
    userInputBox = document.getElementById("user-input-box");

    window.addEventListener("keydown", (e) => {
      handleKeyDown(e);
    });
  });

  listen<EmittedChatMessage>("stream_chat", async (event) => {
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
      await tick();
    } else {
      disableSendButton = false;
      chatHistory[idx].done = true;
    }
  });
</script>

<main class="flex flex-col h-full w-full max-w-7xl">
  <ScrollArea
    id="chat-history"
    class="h-full border border-black m-2 rounded-xl"
  >
    {#each chatHistory as msg, i}
      <EdraEditor
        id={`message-box-${i}`}
        chatMessage={msg}
        index={i}
        {handleInputBoxSelection}
      />
    {/each}
  </ScrollArea>
  <div class="border h-px w-full"></div>
  <div class="m-2 h-fit">
    <Textarea
      id="user-input-box"
      placeholder="Type your message here."
      bind:value={userMessage.content}
      onfocus={() => handleInputBoxSelection(chatHistory.length)}
    />
    <div class="flex flex-row-reverse justify-between w-full py-2">
      <CustomButton
        id="chat-send-btn"
        buttonText="Send"
        onclick={streamChat}
        disable={disableSendButton}
      />
      <p>{currentInputBoxIndex}</p>
      <p>{currentInputBoxIndex === chatHistory.length}</p>

      <ModelSelectionDropdownMenu
        index={modelState.index}
        models={modelState.models}
        {handleModelSelection}
      />
    </div>
  </div>
</main>
