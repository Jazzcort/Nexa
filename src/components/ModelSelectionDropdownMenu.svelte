<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import Icon from "@iconify/svelte";

  let {
    class: className,
    index,
    models,
    handleModelSelection,
  }: {
    class?: string;
    index: number;
    models: string[];
    handleModelSelection: (index: number) => void;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} class={className} variant="ghost" size="sm">
        {models[index]}
        <Icon icon="dashicons:arrow-down" width="20" height="20" />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-56 m-2">
    <DropdownMenu.Group>
      <DropdownMenu.Label>Models</DropdownMenu.Label>
      <DropdownMenu.Separator />
      {#each models as model, i}
        <DropdownMenu.CheckboxItem
          checked={i === index}
          onclick={() => {
            handleModelSelection(i);
          }}>{model}</DropdownMenu.CheckboxItem
        >
      {/each}
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
