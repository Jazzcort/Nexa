<script lang="ts">
  import type { EmittedChatMessage, ChatMessage } from "$lib/type";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { Textarea } from "$lib/components/ui/textarea/index";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import CustomButton from "./CustomButton.svelte";
  import MessageBox from "./MessageBox.svelte";
  // listen<string>("testing", (event) => {
  //   responseText += event.payload;
  // });

  // function display(history: ChatMessage[]): string {
  //   let res = "";
  //   history.forEach((msg) => {
  //     res += msg.content + "\n";
  //   });
  //
  //   return res;
  // }

  listen<EmittedChatMessage>("llm-response", (event) => {
    if (!event.payload.done) {
      chatHistory[chatHistory.length - 1].content +=
        event.payload.message.content;
    } else {
      disableSendButton = false;
      // chatHistory.push({ ...llmResponse });
      // llmResponse.content = "";
    }
  });

  let disableSendButton = $state(false);
  let userMessage: ChatMessage = $state({
    role: "user",
    content: "",
  });
  let chatHistory: ChatMessage[] = $state([
    {
      role: "system",
      content: "PLEASE BE CREATIVE",
    },
  ]);

  const testAsync = () => {
    chatHistory.push({ ...userMessage });
    userMessage.content = "";
    chatHistory.push({
      role: "assistant",
      content: "",
    });
    disableSendButton = true;
    invoke("test_async", { messages: chatHistory });
  };
</script>

<main class="flex flex-col h-full w-full">
  <ScrollArea class="h-full border border-black m-2 rounded-xl" hideScrollBar>
    {#each chatHistory as msg}
      <MessageBox content={msg.content} />
    {/each}
  </ScrollArea>
  <div class="border h-px w-full"></div>
  <div class="m-2 h-fit">
    <Textarea
      placeholder="Type your message here."
      bind:value={userMessage.content}
    />
    <div class="flex flex-row-reverse w-full py-2">
      <CustomButton
        class=""
        buttonText="Send"
        onclick={testAsync}
        disable={disableSendButton}
      />
    </div>
  </div>
</main>
