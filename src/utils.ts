import { invoke } from '@tauri-apps/api';
import { emit } from '@tauri-apps/api/event';
import { pascalCase } from 'change-case';
import type { AppWindow } from '../src-tauri/bindings/AppWindow';
import type { ChannelList } from '../src-tauri/bindings/ChannelList';
import type { LogicalFrame } from '../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalPosition } from '../src-tauri/bindings/geometry/LogicalPosition';
import type { EventUserInteraction } from '../src-tauri/bindings/user_interaction/EventUserInteraction';

export function convert_global_frame_to_local(
	global_frame: LogicalFrame,
	reference_position_global: LogicalPosition
): LogicalFrame {
	return {
		origin: {
			x: global_frame.origin.x - reference_position_global.x,
			y: global_frame.origin.y - reference_position_global.y
		},
		size: {
			width: global_frame.size.width,
			height: global_frame.size.height
		}
	};
}

export async function toggle_main_window_and_rebind_widget(open: boolean) {
	const event: EventUserInteraction = {
		event: 'ToggleMainWindow',
		payload: open
	};
	const channel: ChannelList = 'EventUserInteractions';

	await emit(channel, event);

	// Rebind the MainWindow and WidgetWindow. Because of how MacOS works, we need to have some
	// delay between setting a new position and recreating the parent/child relationship.
	// Pausing the main thread is not possible. Also, running this task async is also not trivial.
	// We send a message to the main thread to run this task.
	// EventWindowControls::RebindMainAndWidget.publish_to_tauri(&app_handle());
	if (open) {
		setTimeout(() => {
			invoke('cmd_rebind_main_widget');
		}, 100);
	}
}

export const get_current_app_window = (): AppWindow => {
	return pascalCase(window.location.pathname.replace('/', '')) as AppWindow;
};

export const format_with_ellipsis_if_too_long = (
	str: string | undefined | null,
	max_length: number
): string => {
	if (str === undefined || str === null) {
		return '';
	}
	if (str.length > max_length) {
		return str.slice(0, max_length) + '…';
	}
	return str;
};
