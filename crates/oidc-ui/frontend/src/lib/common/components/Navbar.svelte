<script lang="ts" module>
	import type { Snippet } from 'svelte';
	import { resolve } from '$app/paths';
	import { Permission } from '../openapi/oidc';
	import { hasPermission, logout } from '../state/currentUser.svelte';
	import { ChevronLeft, LogOut } from '@lucide/svelte';
	import { goto } from '$app/navigation';

	export interface SidebarProps {
		user: CurrentUser;
		children: Snippet<[]>;
	}
</script>

<script lang="ts">
	import { page } from '$app/state';

	let { user, children }: SidebarProps = $props();

	let open = $state(false);

	function toggleOpen() {
		open = !open;
	}
	async function onLogout() {
		logout();
		await goto(resolve('/signin'));
	}
</script>

<div class="flex grow flex-row">
	<nav class={'flex shrink flex-col border-r border-gray-600 bg-gray-100 dark:bg-gray-800'}>
		<div class="flex h-full w-full grow flex-col overflow-auto">
			<div class="m-2 flex shrink flex-row items-center justify-between">
				<a href={resolve('/')} class={{ hidden: open }}><h3 class="m-0">OIDC</h3></a>
				<button
					type="button"
					class="btn icon ghost"
					aria-pressed={!open}
					aria-expanded={open}
					aria-label={open ? 'Collapse sidebar' : 'Expand sidebar'}
					onclick={toggleOpen}
				>
					<ChevronLeft
						class={{ 'text-gray-200 transition-transform duration-200': true, 'rotate-180': open }}
					/>
				</button>
			</div>
			<hr />
			<div class="flex grow">
				<div
					class={{
						'flex flex-col': true,
						hidden: open
					}}
				>
					<a
						class={{
							'btn ghost rounded-none border-0': true,
							active: page.route.id === '/(auth)/(home)/profile'
						}}
						href={resolve('/profile')}
					>
						Profile</a
					>
					{#if hasPermission(user, Permission.Admin)}
						<a
							class={{
								'btn ghost rounded-none border-0': true,
								active: page.route.id === '/(auth)/(home)/config'
							}}
							href={resolve('/config')}>Application Config</a
						>
					{/if}
					{#if hasPermission(user, Permission.ClientWrite) || hasPermission(user, Permission.ClientRead)}
						<a
							class={{
								'btn ghost rounded-none border-0': true,
								active: page.route.id === '/(auth)/(home)/clients'
							}}
							href={resolve('/clients')}>Clients</a
						>
					{/if}
					{#if hasPermission(user, Permission.UserWrite) || hasPermission(user, Permission.UserRead)}
						<a
							class={{
								'btn ghost rounded-none border-0': true,
								active: page.route.id === '/(auth)/(home)/users'
							}}
							href={resolve('/users')}>Users</a
						>
					{/if}
				</div>
			</div>
			<hr />
			<div class="m-2 flex shrink flex-row justify-end">
				<button type="button" class="btn danger icon" onclick={onLogout}>
					<LogOut />
				</button>
			</div>
		</div>
	</nav>
	<main class="flex grow flex-col">
		<div class="h-full w-full overflow-auto">
			<div class="p-4 pb-20">
				<div class="mx-auto max-w-4xl">
					{@render children()}
				</div>
			</div>
		</div>
	</main>
</div>
