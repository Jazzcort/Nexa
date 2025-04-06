<script lang="ts">
  let { class: className, buttonText, onclick, disable } = $props();
  import { Button } from "$lib/components/ui/button/index.js";
</script>

<Button class={className} variant="outline" {onclick} disabled={disable}
  >{buttonText}</Button
>
