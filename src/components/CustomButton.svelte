<script lang="ts">
  let { class: className, buttonText, onclick } = $props();
  import { Button } from "$lib/components/ui/button/index.js";
</script>

<Button class={className} variant="outline" {onclick}>{buttonText}</Button>
