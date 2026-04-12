const DEFAULT_RANGE = "^1.2.3";
const DEFAULT_VERSION = "1.5.0";

type PlaygroundQueryState = {
  rangeInput: string;
  versionInput: string;
};

export function getDefaultRange(): string {
  return DEFAULT_RANGE;
}

export function getDefaultVersion(): string {
  return DEFAULT_VERSION;
}

export function readInputsFromQuery(): PlaygroundQueryState {
  const url = new URL(location.href);

  return {
    rangeInput: url.searchParams.get("range") ?? DEFAULT_RANGE,
    versionInput: url.searchParams.get("version") ?? DEFAULT_VERSION,
  };
}

export function writeInputsToQuery(
  rangeInput: string,
  versionInput: string,
): void {
  const url = new URL(location.href);
  url.searchParams.set("range", rangeInput);
  url.searchParams.set("version", versionInput);
  history.replaceState(null, "", url);
}
