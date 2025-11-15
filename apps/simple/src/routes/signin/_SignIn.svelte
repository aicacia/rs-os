<script lang="ts">
	import { getUserManager, getAuthority, setAuthority } from "$lib/common/state/user.svelte";

	let authority = $state('');

	$effect(() => {
		authority = getAuthority();
	})

	function onChangeAuthority() {
		setAuthority(authority);
	}

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		await getUserManager().signinRedirect();
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<input
		type="text"
		aria-label="OIDC Authority URL"
		autocomplete="url"
		placeholder="OIDC Authority URL"
		bind:value={authority}
		oninput={onChangeAuthority}
	/>
	<input class="btn primary mt-4" type="submit" value="Sign in" />
</form>
