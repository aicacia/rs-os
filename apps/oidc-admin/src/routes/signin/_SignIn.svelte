<script lang="ts">
	import { getIsServicesReady } from '$lib/common/state/services.svelte';
	import { getUserManager } from '$lib/common/state/user.svelte';

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		const userManager = await getUserManager();
		await userManager.signinRedirect();
	}
</script>

<form onsubmit={onSubmit}  class="flex flex-col">
	<input class="btn primary" type="submit" value="Sign in" disabled={!getIsServicesReady()} />
</form>
