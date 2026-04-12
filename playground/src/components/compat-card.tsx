import { Hexagon } from "lucide-preact";
import semver from "semver";

import { buildCompatIssueUrl } from "../utils/issues.ts";
import {
  getCompatLabel,
  getCompatTone,
  type ParseResult,
  type SatisfiesResult,
} from "../utils/result.tsx";
import { ResultCard } from "./result-card.tsx";

type CompatCardProps = {
  rangeInput: string;
  versionInput: string;
  rangeResult: ParseResult | null;
  versionResult: ParseResult | null;
  satisfiesResult: SatisfiesResult | null;
  commitHash?: string;
};

export function CompatCard(
  {
    rangeInput,
    versionInput,
    rangeResult,
    versionResult,
    satisfiesResult,
    commitHash,
  }: CompatCardProps,
) {
  if (
    rangeResult === null || versionResult === null || satisfiesResult === null
  ) {
    return (
      <ResultCard
        title="node-semver compat"
        tone="neutral"
        label="Pending"
        detail="Range parse: pending\nVersion parse: pending\nSatisfies: pending"
        icon={<Hexagon aria-hidden="true" size={16} strokeWidth={2} />}
      />
    );
  }

  const nodeRangeOk = semver.validRange(rangeInput) !== null;
  const nodeVersionOk = semver.valid(versionInput) !== null;
  const nodeSatisfies = getNodeSatisfies(
    nodeRangeOk,
    nodeVersionOk,
    versionInput,
    rangeInput,
  );

  const allMatch = isParseOk(rangeResult) === nodeRangeOk &&
    isParseOk(versionResult) === nodeVersionOk &&
    satisfiesResult.value === nodeSatisfies;

  const issueUrl = buildCompatIssueUrl(rangeInput, versionInput, commitHash);

  return (
    <ResultCard
      title="node-semver compat"
      tone={getCompatTone(allMatch)}
      label={getCompatLabel(allMatch)}
      detail={getCompatDetail(allMatch, issueUrl)}
      icon={<Hexagon aria-hidden="true" size={16} strokeWidth={2} />}
    />
  );
}

function getCompatDetail(allMatch: boolean, issueUrl: string) {
  if (allMatch) {
    return "The result follows node-semver parsing and range semantics.";
  }

  return (
    <>
      Compatibility issues are found in the result. Please report them easily
      using the following link:
      <br />
      <a class="inline-link" href={issueUrl}>GitHub Issues</a>
    </>
  );
}

function getNodeSatisfies(
  nodeRangeOk: boolean,
  nodeVersionOk: boolean,
  versionInput: string,
  rangeInput: string,
) {
  if (nodeRangeOk && nodeVersionOk) {
    return semver.satisfies(versionInput, rangeInput);
  }

  return null;
}

function isParseOk(result: ParseResult): boolean {
  if (result === null) {
    return false;
  }

  return "canonical" in result;
}
