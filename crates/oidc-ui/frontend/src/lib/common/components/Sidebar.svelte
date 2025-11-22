<script lang="ts" module>
	import type { Snippet } from 'svelte';

	export interface SidebarProps {
		nav: Snippet<[]>;
		children: Snippet<[]>;
	}
</script>

<script lang="ts">
	import { ChevronLeft } from '@lucide/svelte';

	let { nav, children }: SidebarProps = $props();

	let collapsed = $state(false);

	function toggleCollapsed() {
		collapsed = !collapsed;
	}
</script>

<div class="flex h-full w-full grow flex-row">
	<nav class={'flex shrink flex-row border-r border-gray-600 bg-gray-100 dark:bg-gray-800'}>
		<ul
			class={{
				'm-0 list-none p-0': true,
				'w-fit opacity-100': !collapsed,
				'w-0 opacity-0': collapsed
			}}
			aria-hidden={collapsed}
		>
			{@render nav()}
		</ul>
		<div class="m-2 flex flex-col">
			<button
				type="button"
				class="btn icon ghost"
				aria-pressed={collapsed}
				aria-expanded={!collapsed}
				aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
				onclick={toggleCollapsed}
			>
				<ChevronLeft
					size={18}
					class={'text-gray-200 transition-transform duration-200' +
						(collapsed ? ' rotate-180' : '')}
				/>
			</button>
		</div>
	</nav>

	<main class="flex grow flex-col p-4">
		{@render children()}
	</main>
</div>
