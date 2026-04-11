const ISSUES_URL = "https://github.com/ryuapp/js-semver/issues/new";

export function buildCompatIssueUrl(
  rangeInput: string,
  versionInput: string,
  commitHash?: string,
): string {
  const bodyLines = [
    "<!-- Please create this issue as-is :) -->",
    "Compatibility issues were found in the playground.",
    "",
    `- Range: \`${rangeInput}\``,
    `- Version: \`${versionInput}\``,
  ];

  if (commitHash && commitHash !== "unknown") {
    bodyLines.push("", `Commit hash: \`${commitHash}\``);
  }

  const searchParams = new URLSearchParams({
    title: `compat: range=${rangeInput} version=${versionInput}`,
    body: bodyLines.join("\n"),
  });

  return `${ISSUES_URL}?${searchParams.toString()}`;
}
