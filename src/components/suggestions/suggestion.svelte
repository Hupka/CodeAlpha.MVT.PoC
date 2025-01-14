<script lang="ts">
	import H3 from '../common/typography/h3.svelte';
	import { emit } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import Card from '../common/card.svelte';
	import H4 from '../common/typography/h4.svelte';
	import Arrow from './icons/arrow.svelte';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import ComplexityBadge from './complexity-badge.svelte';
	import IconProcessing from '../widget/icons/icon-processing.svelte';
	import { fade } from 'svelte/transition';
	import Button from '../common/button/button.svelte';
	import { ButtonType } from '../common/button/button';
	import SuggestionsIcon from './icons/suggestions-icon.svelte';
	import { format_with_ellipsis_if_too_long } from '../../utils';
	import FeedbackBar from '../common/feedback-bar.svelte';

	export let suggestion: FERefactoringSuggestion;
	export let suggestion_id: string;
	export let window_uid: number;
	export let expanded = false;

	$: recalculating = suggestion.state === 'Recalculating';

	const apply_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'PerformSuggestion',
			payload: { id: suggestion_id, editor_window_uid: window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';

		await emit(channel, event);
	};

	const dismiss_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'DismissSuggestion',
			payload: { id: suggestion_id, editor_window_uid: window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';
		await emit(channel, event);
	};

	const EXPANDED_MAX_FUNCTION_NAME_LENGTH = 18;
	const MAX_FUNCTION_NAME_LENGTH = 23;
</script>

<Card on:click additional_class="suggestion relative">
	{#if expanded && recalculating}
		<div
			class="absolute left-0 top-0 z-10 bg-[#ffffffaa] w-full h-full flex flex-col items-center justify-center px-32 saturate(50%)"
			transition:fade|local={{ duration: 200 }}
		>
			<IconProcessing muted={true} />
		</div>
	{/if}
	<header class="max-w-full">
		<div class="flex justify-between w-full">
			<H3>Reduce complexity</H3>
			<SuggestionsIcon />
		</div>
		<p class="text-contrast text-sm max-w-full leading-[1.714]">
			{#if expanded}
				Your function <code
					>{format_with_ellipsis_if_too_long(
						suggestion.main_function_name,
						EXPANDED_MAX_FUNCTION_NAME_LENGTH
					)}</code
				> may be hard to understand due to nested statements. Consider extracting this code block into
				a separate function.
			{:else}
				Refactor deeply nested statements in function <code
					>{format_with_ellipsis_if_too_long(
						suggestion.main_function_name,
						MAX_FUNCTION_NAME_LENGTH
					)}</code
				>
			{/if}
		</p>
	</header>
	{#if expanded}
		<div class="flex flex-col shrink-0 gap-2 pt-2 items-start w-full">
			<H4>Change impact on complexity score</H4>
			<div class="flex items-center gap-1 pb-2">
				<ComplexityBadge complexity={suggestion.prev_complexity} />
				<Arrow />
				<ComplexityBadge complexity={suggestion.new_complexity} />
			</div>
			<FeedbackBar feedback_target={'MethodExtraction'} />
			<div class="flex justify-between w-full items-center pt-2">
				<Button type={ButtonType.Primary} on:click={apply_suggestion}>Extract function</Button>
				<Button type={ButtonType.Secondary} on:click={dismiss_suggestion}>Dismiss</Button>
			</div>
		</div>
	{/if}
</Card>
