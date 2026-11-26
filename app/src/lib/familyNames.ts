// A component family's name as a person reads it, in the window's language (spec 2026-11-26-ux-consolidation-design.md,
// decision 8): "Photo-eye", not `twinova.sensor_diffuse@1`, and no version.
import { m } from "../paraglide/messages.js";
import { family } from "./project";

const NAMES: Record<string, () => string> = {
  belt: m.family_belt,
  belt_curve: m.family_belt_curve,
  chain_transfer: m.family_chain_transfer,
  pusher: m.family_pusher,
  roller: m.family_roller,
  roller_curve: m.family_roller_curve,
  scanner_barcode: m.family_scanner_barcode,
  sensor_color: m.family_sensor_color,
  sensor_diffuse: m.family_sensor_diffuse,
  sensor_laser: m.family_sensor_laser,
  sensor_through_beam: m.family_sensor_through_beam,
  sink: m.family_sink,
  source: m.family_source,
  stopper: m.family_stopper,
  turntable: m.family_turntable,
};

/** The family of a definition key or id, `twinova.sensor_diffuse@1` or `twinova.sensor_diffuse`. */
export const familyOf = (key: string): string => family(key.split("@")[0]);

/** Its name, or the family's own words for one the app does not name yet (a library newer than the window). */
export const familyName = (key: string): string => NAMES[familyOf(key)]?.() ?? familyOf(key).replaceAll("_", " ");

/** The families the window names, for the test that keeps them in step with the library. */
export const namedFamilies = (): string[] => Object.keys(NAMES);
