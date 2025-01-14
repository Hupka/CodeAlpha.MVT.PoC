import { listen } from '@tauri-apps/api/event';
import type { ChannelList } from '../src-tauri/bindings/ChannelList';
import type { LogicalPosition } from '../src-tauri/bindings/geometry/LogicalPosition';
import type { EventWindowControls } from '../src-tauri/bindings/window_controls/EventWindowControls';
import type { MouseButton } from '../src-tauri/bindings/window_controls/MouseButton';
import type { ClickType } from '../src-tauri/bindings/window_controls/ClickType';
import type { TrackingAreaClickedMessage } from '../src-tauri/bindings/window_controls/TrackingAreaClickedMessage';
import type { TrackingAreaMouseOverMessage } from '../src-tauri/bindings/window_controls/TrackingAreaMouseOverMessage';
import { get_current_app_window } from './utils';
import type { AppWindow } from '../src-tauri/bindings/AppWindow';

export class MouseManager {
	elements: HTMLElement[] = [];
	app_window: AppWindow;

	constructor() {}

	async init() {
		await this.listen_mouse_events();
		this.app_window = get_current_app_window();
	}

	private async listen_mouse_events() {
		await listen('EventWindowControls' as ChannelList, (event) => {
			let tracking_event = JSON.parse(event.payload as string) as EventWindowControls;

			if (!this.is_event_for_current_window(tracking_event)) {
				return;
			}

			switch (tracking_event.event) {
				case 'TrackingAreaMouseOver':
					this.mouse_over(tracking_event.payload);
					break;
				case 'TrackingAreaClicked':
					if (tracking_event.payload.app_window === 'CodeOverlay') {
						this.clicked(tracking_event.payload);
					}
					break;
				case 'TrackingAreaClickedOutside':
					break;
				case 'TrackingAreaExited':
					this.area_exited();
					break;
			}
		});
	}

	private mouse_over(msg: TrackingAreaMouseOverMessage) {
		const new_elements = this.elements_from_point(msg.mouse_position);

		const removed_elements = this.elements.filter((element) => !new_elements.includes(element));
		removed_elements.forEach((element) => {
			element.dispatchEvent(new MouseEvent('mouseleave', { bubbles: false })); // For control logic
			element.classList.remove('hover'); // For styling
		});

		const added_elements = new_elements.filter((element) => !this.elements.includes(element));
		added_elements.forEach((element) => {
			element.dispatchEvent(new MouseEvent('mouseenter', { bubbles: false }));
			element.classList.add('hover');
		});

		this.elements = new_elements;
	}

	private clicked(msg: TrackingAreaClickedMessage) {
		document.elementsFromPoint(msg.mouse_position.x, msg.mouse_position.y).forEach((element) => {
			this.simulate_mouse_event_on_element(element, msg.button, msg.click_type);
		});
	}

	private area_exited() {
		this.elements.forEach((element) => {
			element.dispatchEvent(new MouseEvent('mouseleave', { bubbles: false }));
			element.classList.remove('hover');
		});
		this.elements = [];
	}

	private simulate_mouse_event_on_element(
		element: Element,
		button: MouseButton,
		click_type: ClickType
	) {
		let event = 'None';
		switch (click_type) {
			case 'Down':
				event = 'mousedown';
				break;
			case 'Up':
				event = 'mouseup';
				break;
		}

		if (event === 'None' || button != 'Left') {
			return;
		} else {
			element.dispatchEvent(new MouseEvent(event, { bubbles: false }));
		}
	}

	private elements_from_point(position: LogicalPosition): HTMLElement[] {
		return document.elementsFromPoint(position.x, position.y).map((element) => {
			return element as HTMLElement;
		});
	}

	private is_event_for_current_window(tracking_event: EventWindowControls): boolean {
		if (tracking_event.event === 'DarkModeUpdate') {
			return false;
		}

		if (
			tracking_event.event === 'AppWindowHide' ||
			tracking_event.event === 'AppWindowShow' ||
			tracking_event.event === 'AppWindowUpdate'
		) {
			if (tracking_event.payload.app_windows.includes(this.app_window)) {
				return true;
			}
		} else if (tracking_event.payload.app_window === this.app_window) {
			return true;
		}
		return false;
	}
}
