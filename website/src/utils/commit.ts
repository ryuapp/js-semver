export function getCommitLink(commitHash?: string): string {
  if (commitHash) {
    return `https://github.com/ryuapp/js-semver/commit/${commitHash}`;
  }

  return "https://github.com/ryuapp/js-semver";
}

export function getCommitHash(commitHash?: string): string {
  if (commitHash) {
    return commitHash.slice(0, 7);
  }

  return "unknown";
}
