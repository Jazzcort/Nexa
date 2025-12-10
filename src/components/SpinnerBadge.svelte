<script lang="ts">
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Spinner } from "$lib/components/ui/spinner/index.js";
	import type { HTMLAttributes } from "svelte/elements";
	type CustomProps = {
		class?: string;
		msg?: string;
	};
	type RootHtmlProps = HTMLAttributes<HTMLAnchorElement>;
	type Props = CustomProps & RootHtmlProps;
	let { class: className, msg, ...restProps }: Props = $props();
</script>

<Badge variant="outline" class={className} {...restProps}>
	<Spinner />
	{msg}
</Badge>
