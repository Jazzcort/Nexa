<script lang="ts">
  import type {
    EmittedChatMessage,
    ChatMessage,
    ChatMessageWithId,
  } from "$lib/type";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { Textarea } from "$lib/components/ui/textarea/index";
  // import {
  //   DropdownMenu,
  //   DropdownMenuTrigger,
  //   DropdownMenuContent,
  //   DropdownMenuItem,
  //   DropdownMenuGroup,
  // } from "$lib/components/ui/dropdown-menu";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import CustomButton from "./CustomButton.svelte";
  import MessageBox from "./MessageBox.svelte";
  import { onMount, tick } from "svelte";
  import { v4 as uuidv4 } from "uuid";
  import TiptapEditor from "./TiptapEditor.svelte";
  import EdraEditor from "./EdraEditor.svelte";
  import { Button } from "$lib/components/ui/button";
  import ModelSelectionDropdownMenu from "./ModelSelectionDropdownMenu.svelte";
  import { modelState } from "../states/modelState.svelte";

  let scrollDown: HTMLElement | null;
  let chatSendBtn: HTMLElement | null;
  let userInputBox: HTMLElement | null;

  let showStatusBar = $state(true);
  let showActivityBar = $state(false);
  let showPanel = $state(false);

  const handleModelSelection = (index: number) => {
    modelState.index = index;
  };

  onMount(() => {
    chatSendBtn = document.getElementById("chat-send-btn");
    userInputBox = document.getElementById("user-input-box");

    window.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        testAsync();
      }
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

  let disableSendButton = $state(false);
  let userMessage: ChatMessageWithId = $state({
    id: uuidv4(),
    role: "user",
    content: "",
    done: false,
  });
  let chatHistory: ChatMessageWithId[] = $state([]);

  const testAsync = async () => {
    chatHistory.push({ ...userMessage, done: true });
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
      userMessage.id = uuidv4();
      userInputBox?.focus();
    }, 80);
  };
</script>

<main class="flex flex-col h-full w-full max-w-7xl">
  <ScrollArea
    id="chat-history"
    class="h-full border border-black m-2 rounded-xl"
  >
    {#each chatHistory as msg, i}
      <!-- <TiptapEditor id={`message-box-${i}`} content={msg.content} /> -->
      <EdraEditor id={`message-box-${i}`} chatMessage={msg} />
      <!-- <MessageBox id={`message-box-${i}`} content={msg.content} /> -->
    {/each}
  </ScrollArea>
  <div class="border h-px w-full"></div>
  <div class="m-2 h-fit">
    <Textarea
      id="user-input-box"
      placeholder="Type your message here."
      bind:value={userMessage.content}
    />
    <div class="flex flex-row-reverse justify-between w-full py-2">
      <CustomButton
        id="chat-send-btn"
        buttonText="Send"
        onclick={testAsync}
        disable={disableSendButton}
      />
      <ModelSelectionDropdownMenu
        index={modelState.index}
        models={modelState.models}
        {handleModelSelection}
      />
    </div>
  </div>
</main>
