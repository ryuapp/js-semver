const ISSUES_URL = "https://github.com/ryuapp/js-semver/issues/new";

export function buildCompatIssueUrl(
  rangeInput: string,
  versionInput: string,
): string {
  const searchParams = new URLSearchParams({
    title: `compat: range=${rangeInput} version=${versionInput}`,
    body: [
      "<!-- Please create this issue as-is :) -->",
      "Compatibility issues were found in the playground.",
      "",
      `- Range: \`${rangeInput}\``,
      `- Version: \`${versionInput}\``,
    ].join("\n"),
  });

  return `${ISSUES_URL}?${searchParams.toString()}`;
}
