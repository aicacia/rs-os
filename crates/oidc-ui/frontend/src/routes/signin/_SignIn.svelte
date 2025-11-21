<script lang="ts">
	import { signInUsernamePassword } from '$lib/common/state/currentUser.svelte';

	let username = $state('');
	let password = $state('');

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		await signInUsernamePassword(username, password);
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<label class="flex flex-col"
		>Username
		<input
			type="text"
			aria-label="Username"
			autocomplete="username"
			placeholder="Enter Username"
			bind:value={username}
		/>
	</label>
	<label class="flex flex-col"
		>Password
		<input
			aria-label="Password"
			type="password"
			autocomplete="current-password"
			placeholder="Enter Password"
			bind:value={password}
		/>
		<input class="btn primary mt-4" type="submit" value="Sign in" />
	</label>
</form>
