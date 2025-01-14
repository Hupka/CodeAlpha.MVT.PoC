// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Annotation } from "./Annotation";
import type { FeatureKind } from "./FeatureKind";

export interface AnnotationGroup {
  id: string;
  editor_window_uid: number;
  feature: FeatureKind;
  annotations: Record<string, Annotation>;
}
