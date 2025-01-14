// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AppWindow } from "../AppWindow";
import type { CodeDocumentFrameProperties } from "../macOS_specific/xcode/CodeDocumentFrameProperties";
import type { LogicalPosition } from "../geometry/LogicalPosition";
import type { LogicalSize } from "../geometry/LogicalSize";
import type { ViewportProperties } from "../macOS_specific/xcode/ViewportProperties";

export interface UpdateAppWindowMessage {
  app_windows: Array<AppWindow>;
  viewport: ViewportProperties | null;
  code_document: CodeDocumentFrameProperties | null;
  window_position: LogicalPosition | null;
  window_size: LogicalSize | null;
}
