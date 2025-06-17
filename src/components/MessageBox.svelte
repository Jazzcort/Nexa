<script>
  let props = $props();

  $effect(() => {
    // console.log(props.content);
  });
</script>

<div
  id={props.id}
  class="border m-2 rounded-sm border-black p-2 whitespace-pre-wrap"
>
  {props.content}
</div>
