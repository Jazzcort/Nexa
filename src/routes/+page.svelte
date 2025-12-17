<script lang="ts">
  import type {
    EmittedChatMessage,
    ChatMessageWithId,
    ChatMessage,
    FunctionCallRequest,
    FunctionCallInfo,
    EmittedMCPResponse,
    Text,
    UserChatMessage,
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
  import FunctionCallCell from "$components/FunctionCallCell.svelte";
  import SpinnerBadge from "$components/SpinnerBadge.svelte";
  import { ReactiveFunctionCallInfo } from "$lib/stores/chat-history.svelte";
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
  let userMessage: string = $state("");

  let awaitingFunctionCalls: string[] = $state([]);

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

  const scrollToBottom = async () => {
    await tick();
    if (scrollDown) {
      scrollDown.scrollIntoView({
        behavior: "instant",
        block: "end",
      });
    }
  };

  const assignScrollDownElement = async () => {
    await tick();
    scrollDown = document.getElementById(
      `message-box-${chatHistoryStore.chatHistory.length - 1}`,
    );
  };

  const triggerStreamChat = (
    index: number,
    modifiedContent: ChatMessageWithId,
  ) => {
    if (index < 0 || index > chatHistoryStore.chatHistory.length || streaming) {
      return;
    }

    chatHistoryStore.chatHistory = [
      ...chatHistoryStore.chatHistory.slice(0, index),
    ];
    chatHistoryStore.chatHistory.push(modifiedContent);
    chatHistoryStore.chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: {
        type: "text",
        content: {
          text: "",
        },
      },
      done: false,
    });

    deleteAwaitingFunctionCalls();
    streamChat();
  };

  const streamChat = async () => {
    streaming = true;
    invoke("stream_chat", {
      history: { messages: chatHistoryStore.chatHistory },
      model: modelState.models[modelState.index].modelId,
      provider: modelState.models[modelState.index].provider,
    });

    chatHistoryStore.saveChatHistory();

    await assignScrollDownElement();

    scrollToBottom();
    setTimeout(() => {
      if (currentInputBoxIndex === chatHistoryStore.chatHistory.length) {
        userMessage = "";
      }
      userInputBox?.focus();
    }, 80);
  };

  const normalUserInput = () => {
    if (
      chatHistoryStore.chatHistory.length !== currentInputBoxIndex ||
      !userMessage.trim()
    ) {
      return;
    }

    chatHistoryStore.chatHistory.push({
      role: "user",
      content: {
        type: "text",
        content: {
          text: userMessage,
        },
      },
      id: uuidv4(),
      done: true,
    });
    chatHistoryStore.chatHistory.push({
      id: uuidv4(),
      role: "assistant",
      content: {
        type: "text",
        content: {
          text: "",
        },
      },
      done: false,
    });

    cancelAwaitingFunctionCalls();
    handleInputBoxSelection(chatHistoryStore.chatHistory.length);
    streamChat();
  };

  const cancelAwaitingFunctionCalls = () => {
    awaitingFunctionCalls.forEach((id) => {
      const functionCall = chatHistoryStore.functionCallInfo.get(id);
      if (functionCall) {
        functionCall.status = "cancelled";
      }
    });
    chatHistoryStore.saveFunctionCallInfo();
    awaitingFunctionCalls = [];
  };

  const deleteAwaitingFunctionCalls = () => {
    awaitingFunctionCalls.forEach((id) => {
      chatHistoryStore.functionCallInfo.delete(id);
    });
    chatHistoryStore.saveFunctionCallInfo();
    awaitingFunctionCalls = [];
  };

  const injectFunctionCalls = async () => {
    const finalIndexOfAssistantTextMessage =
      chatHistoryStore.chatHistory.findLastIndex((msg) => {
        return msg.role === "assistant" && msg.content.type === "text";
      });

    if (finalIndexOfAssistantTextMessage > 0) {
      let emptyResponse: ChatMessageWithId[] = [];
      let toInject: ChatMessageWithId[] = awaitingFunctionCalls.map(
        (functionCallId) => {
          const functionCall =
            chatHistoryStore.functionCallInfo.get(functionCallId);

          emptyResponse.push({
            id: functionCall!.responseId,
            role: "user",
            content: {
              type: "functionCallResponse",
              content: {
                name: functionCall!.functionCall.name,
                id: functionCall!.functionCall.id,
                response: {},
              },
            },
            done: true,
          });
          return {
            id: functionCall!.id,
            role: "assistant",
            content: {
              type: "functionCallRequest",
              content: {
                ...functionCall!.functionCall,
                args: { ...functionCall!.functionCall.args },
              },
            },
            done: true,
          };
        },
      );
      chatHistoryStore.chatHistory.splice(
        finalIndexOfAssistantTextMessage,
        0,
        ...[...toInject, ...emptyResponse],
      );

      await assignScrollDownElement();
    }
  };

  const searchFunctionCallResponse = (responseId: string | undefined) => {
    if (!responseId) {
      return;
    }

    const responseMsg = chatHistoryStore.chatHistory.find(
      (msg) => msg.id === responseId,
    );
    if (!responseMsg || responseMsg.content.type !== "functionCallResponse") {
      return;
    }

    return responseMsg.content.content.response;
  };

  const isAllAwaitingFunctionCallsExecuted = (): boolean => {
    return !awaitingFunctionCalls.some((id) =>
      ["awaiting", "initialized"].includes(
        chatHistoryStore.functionCallInfo.get(id)!.status,
      ),
    );
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
        // chatHistory = chatHistoryStore.chatHistory;
        chatHistoryStore.chatHistory.forEach((msg) => {
          if (
            msg.content.type === "functionCallRequest" &&
            chatHistoryStore.functionCallInfo.get(msg.id)?.status ===
              "initialized"
          ) {
            awaitingFunctionCalls.push(msg.id);
          }
        });
        didLoadChatHistory = true;
        clearInterval(interval);
      }
    }, 100);

    let unlistenToStreamChat: UnlistenFn | undefined = undefined;
    let unlistenToMCPResponse: UnlistenFn | undefined = undefined;

    async function listenToStreamChat() {
      unlistenToStreamChat = await listen<EmittedChatMessage>(
        "stream_chat",
        async (event) => {
          if (chatHistoryStore.chatHistory.length <= 0) {
            return;
          }

          const idx = chatHistoryStore.chatHistory.length - 1;
          if (chatHistoryStore.chatHistory[idx].id !== event.payload.id) {
            console.log("not correct id");
            return;
          }

          if (!event.payload.done) {
            const functionCallInfo: ReactiveFunctionCallInfo[] =
              event.payload.message.flatMap((msg) => {
                if (msg.content.type === "functionCallRequest") {
                  const id = uuidv4();
                  awaitingFunctionCalls.push(id);

                  const [functionName, serverName] = msg.content.content.name
                    .split("-_-")
                    .reverse();

                  return [
                    new ReactiveFunctionCallInfo({
                      id,
                      functionCall: msg.content.content,
                      status: "initialized",
                      responseId: uuidv4(),
                      serverName: serverName || "",
                      functionName: functionName || "",
                    }),
                  ];
                } else {
                  return [];
                }
              });

            chatHistoryStore.addFunctionCallInfo(functionCallInfo);

            let textOutput = event.payload.message
              .filter((msg) => msg.content.type === "text")
              .map((msg) => {
                return (msg.content.content as Text).text;
              });

            if (chatHistoryStore.chatHistory[idx].content.type !== "text") {
              console.log("Error: Last chat message is not text content");
              return;
            }

            (chatHistoryStore.chatHistory[idx].content.content as Text).text +=
              textOutput.join("");
          } else {
            streaming = false;
            chatHistoryStore.chatHistory[idx].done = true;

            if (awaitingFunctionCalls.length > 0) {
              await injectFunctionCalls();
            }

            chatHistoryStore.saveChatHistory();
          }

          if (isNearBottom) {
            scrollToBottom();
          }
        },
      );
    }

    async function listenToMCPResponse() {
      unlistenToMCPResponse = await listen<EmittedMCPResponse>(
        "mcp_response",
        async (event) => {
          // Update response in chat history
          const response_msg = chatHistoryStore.chatHistory.find(
            (msg) => msg.id === event.payload.responseId,
          );
          if (
            response_msg &&
            response_msg.content.type === "functionCallResponse"
          ) {
            const response = event.payload.response;
            if ("result" in response) {
              if ("structuredContent" in response.result) {
                response_msg.content.content.response =
                  response.result.structuredContent;
              } else {
                let text = "";
                (response.result.content as any[]).forEach((content) => {
                  if ("text" in content) {
                    text += content.text;
                  }
                });

                response_msg.content.content.response = { result: text };
              }
            } else if ("error" in response) {
              response_msg.content.content.response = { error: response.error };
            }
          }
          chatHistoryStore.saveChatHistory();

          // Update the status
          const functionCall = chatHistoryStore.functionCallInfo.get(
            event.payload.requestId,
          );
          if (functionCall) {
            functionCall.status = "success";
          }

          chatHistoryStore.saveFunctionCallInfo();

          // Trigger stream chat if all the awaiting function calls ran
          if (isAllAwaitingFunctionCallsExecuted()) {
            awaitingFunctionCalls = [];
            streamChat();
          }
        },
      );
    }

    listenToStreamChat();
    listenToMCPResponse();
    invoke("initialize_mcp_client");

    return () => {
      if (userInputBox) {
        userInputBox.removeEventListener("keydown", handleKeyDown);
      }
      if (scrollingArea) {
        scrollingArea.removeEventListener("scroll", checkScrollPosition);
      }
      if (unlistenToStreamChat) {
        unlistenToStreamChat();
      }
      if (unlistenToMCPResponse) {
        unlistenToMCPResponse();
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
        {#each chatHistoryStore.chatHistory as msg, i}
          <div id={`message-box-${i}`}>
            {#if msg.content.type === "text"}
              {#if msg.role === "assistant" && i === chatHistoryStore.chatHistory.length - 1 && msg.content.content.text.trim() === ""}
                {#if streaming}
                  <div class="flex">
                    <div class="flex-1"></div>
                    <SpinnerBadge
                      class="m-2"
                      msg={isAllAwaitingFunctionCallsExecuted()
                        ? "Thinking..."
                        : "Awaiting..."}
                    />
                  </div>
                {/if}
              {:else}
                <TipTapEditor
                  content={msg}
                  index={i}
                  {handleInputBoxSelection}
                  {triggerStreamChat}
                />
              {/if}
            {:else if msg.content.type === "functionCallRequest"}
              {@const functionCall = chatHistoryStore.functionCallInfo.get(
                msg.id,
              )}
              <div class="flex">
                <div class="flex-1"></div>
                <FunctionCallCell
                  class="m-2 w-[350px] space-y-2  py-2 border border-black rounded-md px-2"
                  awaitingFunctionCall={functionCall}
                  functionCallResponse={searchFunctionCallResponse(
                    functionCall?.responseId,
                  )}
                />
              </div>
            {/if}
          </div>
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

  <!-- Test purpose! -->
  <div>{currentInputBoxIndex}</div>
  <div>{JSON.stringify(awaitingFunctionCalls)}</div>

  <div class="m-2 flex flex-col min-h-[120px]">
    <Textarea
      id="user-input-box"
      placeholder="Type your message here."
      bind:value={userMessage}
      onfocus={() => {
        handleInputBoxSelection(chatHistoryStore.chatHistory.length);
      }}
    />
    <div class="flex flex-row-reverse justify-between w-full py-2">
      <Button
        id="chat-send-btn"
        onclick={normalUserInput}
        disabled={streaming || !didLoadChatHistory}>send</Button
      >
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
