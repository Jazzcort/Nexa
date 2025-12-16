<script>
	import "../app.css";
	import { invoke } from "@tauri-apps/api/core";

	let { children } = $props();
</script>

{@render children()}
