const DEFAULT_RANGE = "^1.2.3";
const DEFAULT_VERSION = "1.5.0";

export function getDefaultRange(): string {
  return DEFAULT_RANGE;
}

export function getDefaultVersion(): string {
  return DEFAULT_VERSION;
}

export function readInputsFromQuery(): {
  rangeInput: string;
  versionInput: string;
} {
  const searchParams = new URLSearchParams(location.search);
  const rangeInput = searchParams.get("range");
  const versionInput = searchParams.get("version");

  return {
    rangeInput: rangeInput ?? DEFAULT_RANGE,
    versionInput: versionInput ?? DEFAULT_VERSION,
  };
}

export function writeInputsToQuery(
  rangeInput: string,
  versionInput: string,
): void {
  const searchParams = new URLSearchParams();
  searchParams.set("range", rangeInput);
  searchParams.set("version", versionInput);
  const search = searchParams.toString();
  const nextUrl = `${location.pathname}${search ? `?${search}` : ""}`;
  history.replaceState(null, "", nextUrl);
}
