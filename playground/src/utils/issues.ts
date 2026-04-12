export function buildCompatIssueUrl(
  rangeInput: string,
  versionInput: string,
  commitHash?: string,
): string {
  const title = `Compat issue: range=${rangeInput} version=${versionInput}`;
  const bodyLines = [
    "<!-- Please create this issue as-is :) -->",
    "Compatibility issues were found in the playground.",
    "",
    `- Range: \`${rangeInput}\``,
    `- Version: \`${versionInput}\``,
  ];

  if (commitHash) {
    bodyLines.push("", `Commit hash: \`${commitHash}\``);
  }

  const query = new URLSearchParams({
    title,
    body: bodyLines.join("\n"),
  });

  return `https://github.com/ryuapp/js-semver/issues/new?${query.toString()}`;
}
