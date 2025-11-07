<script lang="ts">
	import { signInUsernamePassword } from '$lib/common/state/currentUser.svelte';

	let username = $state('');
	let password = $state('');

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		const user = await signInUsernamePassword(username, password);
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<input
		type="text"
		aria-label="Username"
		autocomplete="username"
		placeholder="Username"
		bind:value={username}
	/>
	<input
		aria-label="Password"
		type="password"
		autocomplete="current-password"
		placeholder="Password"
		bind:value={password}
	/>
	<input class="btn primary mt-4" type="submit" value="Sign in" />
</form>
