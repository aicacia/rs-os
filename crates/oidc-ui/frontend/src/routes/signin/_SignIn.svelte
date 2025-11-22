<script lang="ts">
	import { signInUsernamePassword } from '$lib/common/state/currentUser.svelte';
	import { m } from '$lib/paraglide/messages';

	let username = $state('');
	let password = $state('');

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		await signInUsernamePassword(username, password);
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<label class="flex flex-col">
		{m.signin_username_label()}
		<input
			type="text"
			aria-label={m.signin_username_label()}
			autocomplete="username"
			placeholder={m.signin_username_placeholder()}
			bind:value={username}
		/>
	</label>
	<label class="flex flex-col">
		{m.signin_password_label()}
		<input
			aria-label={m.signin_password_label()}
			type="password"
			autocomplete="current-password"
			placeholder={m.signin_password_placeholder()}
			bind:value={password}
		/>
		<input class="btn primary mt-4" type="submit" value={m.sign_in()} />
	</label>
</form>
