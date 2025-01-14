// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AppWindow } from "../AppWindow";
import type { ClickType } from "./ClickType";
import type { LogicalPosition } from "../geometry/LogicalPosition";
import type { MouseButton } from "./MouseButton";

export interface TrackingAreaClickedMessage {
  id: string;
  window_uid: number;
  app_window: AppWindow;
  mouse_position: LogicalPosition;
  button: MouseButton;
  click_type: ClickType;
  duration_ms: bigint;
}
