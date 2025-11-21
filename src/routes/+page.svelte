<script lang="ts">
  import type {
    EmittedChatMessage,
    ChatMessageWithId,
    ChatMessage,
  } from "$types";
  import { onDestroy, onMount, tick } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import { Textarea } from "$lib/components/ui/textarea/index";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Button } from "$lib/components/ui/button/index";
  import TipTapEditor from "$components/TipTapEditor/index.svelte";
  import { v4 as uuidv4 } from "uuid";
  import { modelState } from "$states/ollamaModelState.svelte";
  import DropdownMenu from "$components/DropdownMenu.svelte";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import { goto } from "$app/navigation";
  import { chatHistoryStore } from "$lib/stores/chat-history.svelte";
  //
  let scrollTop = $state(0);
  let scrollDown: HTMLElement | null;
  let chatSendBtn: HTMLElement | null;
  let userInputBox: HTMLElement | null;
  let scrollingArea: HTMLElement | null;
  let streaming = $state(false);
  let isNearBottom = $state(true);
  let didLoadChatHistory = $state(false);
  const SCROLL_THRESHOLD = 100;
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

  const checkScrollPosition = () => {
    if (!scrollingArea) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollingArea;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
    isNearBottom = distanceFromBottom <= SCROLL_THRESHOLD;
  };

  const scrollToBottom = () => {
    if (scrollDown) {
      scrollDown.scrollIntoView({
        behavior: "instant",
        block: "end",
      });
    }
  };

  const triggerStreamChat = (
    index: number,
    modifiedContent: ChatMessageWithId,
  ) => {
    console.log(
      modifiedContent.content,
      "modified content in trigger stream chat!!!",
    );

    if (index < 0 || index > chatHistoryStore.chatHistory.length || streaming) {
      return;
    }

    chatHistory = [...chatHistory.slice(0, index)];
    chatHistory.push(modifiedContent);
    chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: "",
      done: false,
    });

    streamChat();
  };

  const streamChat = async () => {
    streaming = true;
    invoke("stream_chat", {
      history: { messages: chatHistory },
      model: modelState.models[modelState.index].modelId,
      provider: modelState.models[modelState.index].provider,
    });

    chatHistoryStore.sync(chatHistory);

    await tick();
    scrollDown = document.getElementById(
      `message-box-${chatHistory.length - 1}`,
    );

    scrollToBottom();
    setTimeout(() => {
      if (currentInputBoxIndex === chatHistory.length) {
        userMessage.content = "";
      }
      userInputBox?.focus();
    }, 80);
  };

  const normalUserInput = () => {
    if (
      chatHistory.length !== currentInputBoxIndex ||
      !userMessage.content.trim()
    ) {
      return;
    }

    chatHistory.push({
      ...userMessage,
      id: uuidv4(),
      done: true,
    });
    chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: "",
      done: false,
    });

    handleInputBoxSelection(chatHistory.length);
    streamChat();
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "Enter":
        if (e.shiftKey) {
          // Add a new line, do nothing here
        } else {
          normalUserInput();
        }

        break;
    }
  };

  onMount(() => {
    chatSendBtn = document.getElementById("chat-send-btn");
    userInputBox = document.getElementById("user-input-box");
    scrollingArea = document.querySelector(
      "#chat-history [data-slot='scroll-area-viewport']",
    ) as HTMLElement;

    if (scrollingArea) {
      scrollingArea.addEventListener("scroll", checkScrollPosition);
    }

    if (userInputBox) {
      userInputBox.addEventListener("keydown", handleKeyDown);
    }

    const interval = setInterval(() => {
      if (chatHistoryStore.isReady) {
        chatHistory = chatHistoryStore.chatHistory;
        didLoadChatHistory = true;
        clearInterval(interval);
      }
    }, 100);

    let unlisten: UnlistenFn | undefined = undefined;

    async function listenToStreamChat() {
      unlisten = await listen<EmittedChatMessage>(
        "stream_chat",
        async (event) => {
          console.log(event.payload.message);
          console.log(event.payload.id);
          console.log(event.payload.done);

          if (chatHistory.length <= 0) {
            return;
          }

          const idx = chatHistory.length - 1;
          if (chatHistory[idx].id !== event.payload.id) {
            console.log("not correct id");
            return;
          }

          if (!event.payload.done) {
            chatHistory[idx].content += event.payload.message.content;

            if (isNearBottom) {
              scrollToBottom();
            }
            await tick();
          } else {
            streaming = false;
            chatHistory[idx].done = true;
            chatHistoryStore.sync(chatHistory);
          }
        },
      );
    }

    listenToStreamChat();

    return () => {
      if (userInputBox) {
        userInputBox.removeEventListener("keydown", handleKeyDown);
      }
      if (scrollingArea) {
        scrollingArea.removeEventListener("scroll", checkScrollPosition);
      }
      if (unlisten) {
        unlisten();
      }
    };
  });
</script>

<main class="app-container">
  <div class="flex-1 flex flex-col mx-2 overflow-hidden">
    <div class="h-[5px] w-full"></div>
    <ScrollArea
      id="chat-history"
      class="flex flex-1 overflow-hidden border border-black rounded-xl"
    >
      {#if didLoadChatHistory}
        {#each chatHistory as msg, i}
          <TipTapEditor
            id={`message-box-${i}`}
            content={msg}
            index={i}
            {handleInputBoxSelection}
            {triggerStreamChat}
          />
        {/each}
      {:else}
        <div class="h-full w-full justify-center items-center flex">
          <Item.Root>
            <Item.Content>
              <Item.Title>{"Loading..."}<Spinner /></Item.Title>
            </Item.Content>
          </Item.Root>
        </div>
      {/if}
    </ScrollArea>
    <div class="h-[5px] w-full"></div>
  </div>
  <!-- <div class="border h-px w-full"></div> -->
  <div class="m-2 flex flex-col min-h-[120px]">
    <Textarea
      id="user-input-box"
      placeholder="Type your message here."
      bind:value={userMessage.content}
      onfocus={() => {
        handleInputBoxSelection(chatHistory.length);
      }}
    />
    <div class="flex flex-row-reverse justify-between w-full py-2">
      <Button
        id="chat-send-btn"
        onclick={normalUserInput}
        disabled={streaming || !didLoadChatHistory}>send</Button
      >
      <!-- Testing purpose -->
      <!-- <p>{currentInputBoxIndex}</p> -->
      <!-- <p>{chatHistory.length} total length</p> -->
      <!-- <p>{currentInputBoxIndex === chatHistory.length}</p> -->
      <Button onclick={() => goto("/config")}>Config</Button>

      <DropdownMenu
        index={modelState.index}
        content={modelState.models}
        handleSelection={handleModelSelection}
      />
    </div>
  </div>
</main>

<style>
  :global(body) {
    display: flex;
    flex-direction: column;
    height: 100vh;
    margin: 0;
  }
  /* Make your app container grow to fill the space */
  .app-container {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
