import initWasm, {
  parse_range,
  parse_version,
  satisfies as satisfies_range,
} from "../../wasm/pkg/js_semver_website_wasm.js";

export function initJsSemver() {
  return initWasm();
}

export function parseRange(input: string): string {
  return parse_range(input);
}

export function parseVersion(input: string): string {
  return parse_version(input);
}

export function satisfies(
  rangeInput: string,
  versionInput: string,
): boolean | null {
  return satisfies_range(rangeInput, versionInput) ?? null;
}

export function getInitErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return "Failed to initialize wasm";
}
