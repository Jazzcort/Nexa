<script lang="ts">
  let {
    class: className,
    buttonText,
    onclick,
    disable,
    id,
  }: {
    class?: string;
    buttonText: string;
    onclick: (() => Promise<void>) | (() => void);
    disable: boolean;
    id: string;
  } = $props();
  import { Button } from "$lib/components/ui/button/index.js";
</script>

<Button {id} class={className} variant="outline" {onclick} disabled={disable}
  >{buttonText}</Button
>
