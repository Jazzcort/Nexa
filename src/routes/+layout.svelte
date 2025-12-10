<script>
	import "../app.css";
	import { invoke } from "@tauri-apps/api/core";

	let { children } = $props();

	invoke("initialize_mcp_client");
</script>

{@render children()}
