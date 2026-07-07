import type { JsonNull } from './json-null';
import type { JsonObject } from './json-object';

/** JSON value accepted by flexible Claw Router metadata and extension maps. */
export type JsonValue = string | number | number | boolean | JsonValue[] | JsonObject | JsonNull;
