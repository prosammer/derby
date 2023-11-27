<script lang="ts">
	import '../app.pcss';
	import '../app.postcss';
	import { onMount } from 'svelte';
	import { appWindow } from '@tauri-apps/api/window';

	let darkMode = false;
	onMount(async () => {
		darkMode = (await appWindow.theme()) === 'dark';
	});
</script>

<div id="darkModeDiv" class="bg-transparent" class:dark={darkMode}>
	<slot />
</div>
