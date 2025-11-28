<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models';
	import { Edit, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		users: User[];
		onEdit?: (user: User) => void;
		onDelete?: (user: User) => void;
	}
	let { users = [], onEdit, onDelete }: Props = $props();

	let sortAsc = $state(true);
	const sorted = $derived.by(() => {
		return [...users].sort((a, b) =>
			sortAsc ? a.username.localeCompare(b.username) : b.username.localeCompare(a.username)
		);
	});
</script>

<div class="overflow-x-auto">
	<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
		<thead class="bg-gray-100 dark:bg-gray-800">
			<tr>
				<th class="px-4 py-2 text-left text-sm font-medium">{m.users_id()}</th>
				<th class="px-4 py-2 text-left text-sm font-medium">
					<button class="flex items-center gap-1" onclick={() => (sortAsc = !sortAsc)}>
						{m.users_username()}
						<span class="text-xs">{sortAsc ? '▲' : '▼'}</span>
					</button>
				</th>
				<th class="px-4 py-2 text-left text-sm font-medium">{m.users_created_at()}</th>
				<th class="px-4 py-2 text-left text-sm font-medium">{m.users_updated_at()}</th>
				<th class="px-4 py-2 text-left text-sm font-medium">{m.users_actions()}</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-gray-200 dark:divide-gray-700">
			{#each sorted as u}
				<tr class="hover:bg-gray-50 dark:hover:bg-gray-800">
					<td class="px-4 py-2 text-sm">{u.id}</td>
					<td class="px-4 py-2 text-sm">{u.username}</td>
					<td class="px-4 py-2 text-sm">{new Date(u.createdAt).toLocaleString()}</td>
					<td class="px-4 py-2 text-sm">{new Date(u.updatedAt).toLocaleString()}</td>
					<td class="px-4 py-2 text-sm">
						<div class="flex gap-2">
							{#if onEdit}
								<button class="btn light" onclick={() => onEdit?.(u)} aria-label="edit">
									<Edit class="h-4 w-4" />
								</button>
							{/if}
							{#if onDelete}
								<button class="btn danger" onclick={() => onDelete?.(u)} aria-label="delete">
									<Trash2 class="h-4 w-4" />
								</button>
							{/if}
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
