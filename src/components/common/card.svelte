<script lang="ts">
	export let id: string | undefined = undefined;
	export let additional_class: string = '';
</script>

<div
	on:click
	{id}
	class="rounded-xl bg-background overflow-hidden py-4 px-4 flex flex-col items-start gap-3 border border-transparent shrink-0 {additional_class} shadow"
>
	<slot />
</div>
