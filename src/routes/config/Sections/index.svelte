<script lang="ts">
	import APIKeysSection from "./APIKeysSection.svelte";
	import GeneralSection from "./GeneralSection.svelte";
	import type { ConfigSection } from "$types";
	import type { Component } from "svelte";

	let SelectedSection: Component = $derived.by(() => {
		switch (section) {
			case "general":
				return GeneralSection;
			case "apiKeys":
				return APIKeysSection;
		}
	});

	let { section }: { section: ConfigSection } = $props();
</script>

<SelectedSection />
