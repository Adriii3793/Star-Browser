<script lang="ts">
	import Loading from '../ui/Loading.svelte';
	import { PRESET_AVATARS } from '$lib/data/presets';
	import { setup } from '$lib/stores/setup.svelte';

	let { ondone }: { ondone: () => void } = $props();

	let done = $state(0);
	let total = $state(1);
	let progress = $derived(total > 0 ? Math.round((done / total) * 100) : 0);

	function preloadImage(src: string): Promise<void> {
		return new Promise((resolve) => {
			const img = new Image();
			img.onload = () => resolve();
			img.onerror = () => {
				console.warn('Preset avatar missing, skipping:', src);
				resolve();
			};
			img.src = src;
		});
	}

	async function boot() {
		const tasks: Promise<unknown>[] = [...PRESET_AVATARS.map(preloadImage), setup.load()];

		total = tasks.length;
		done = 0;
		await Promise.all(tasks.map((t) => t.then(() => (done += 1))));

		ondone();
	}

	$effect(() => {
		boot();
	});
</script>

<div class="wrap">
	<Loading size={96} showText={false} />
	<p class="label">Preparing your browser</p>
	<div class="bar">
		<div class="fill" style="width:{progress}%"></div>
	</div>
</div>

<style>
	.wrap {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 18px;
		height: 100%;
	}

	.label {
		margin: 0;
		font-size: 14px;
		color: var(--text-soft, #8a6b57);
	}

	.bar {
		width: 240px;
		height: 4px;
		border-radius: 999px;
		background: var(--field, #f7f1ec);
		overflow: hidden;
	}

	.fill {
		height: 100%;
		background: var(--accent, #e8734a);
		transition: width 0.18s linear;
	}

	@media (prefers-reduced-motion: reduce) {
		.fill {
			transition: none;
		}
	}
</style>
