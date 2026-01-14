<script lang="ts">
	import { getOSBaseUrl, setOSBaseUrl } from "$lib/common/state/services.svelte";
	import { getUserManager } from "$lib/common/state/user.svelte";

	let osBaseUrl = $state(getOSBaseUrl());

	$effect(() => {
		osBaseUrl = getOSBaseUrl();
	})

	function onChangeOSBaseUrl() {
		setOSBaseUrl(osBaseUrl);
	}

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		
		const userManager = await getUserManager();
		await userManager.signinRedirect();
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<input
		type="text"
		aria-label="OS Base URL"
		autocomplete="url"
		placeholder="OS Base URL"
		bind:value={osBaseUrl}
		oninput={onChangeOSBaseUrl}
	/>
	<input class="btn primary mt-4" type="submit" value="Sign in" />
</form>
