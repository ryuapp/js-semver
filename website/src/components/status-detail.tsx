import type {
  ParseResult,
  SatisfiesResult,
  StatusResult,
} from "../utils/result.tsx";
import { getParseStatus, getSatisfiesStatus } from "../utils/result.tsx";

type RangeStatusDetailProps = {
  result: ParseResult | null;
};

export function RangeStatusDetail(
  { result }: RangeStatusDetailProps,
) {
  const status = getParseStatus(result);

  return (
    <div class="grid gap-1">
      <div class={getStatusClassName(status.tone)}>
        <span>{status.content}</span>
      </div>
      <div class="hidden" aria-hidden="true">
        <span>1.0.0 satisfies *</span>
      </div>
    </div>
  );
}

type VersionStatusDetailProps = {
  parseResult: ParseResult | null;
  satisfiesResult: SatisfiesResult | null;
};

export function VersionStatusDetail(
  { parseResult, satisfiesResult }: VersionStatusDetailProps,
) {
  const parseStatus = getParseStatus(parseResult);
  const satisfiesStatus = getSatisfiesStatus(satisfiesResult);

  return (
    <div class="grid gap-1">
      <div class={getStatusClassName(parseStatus.tone)}>
        <span>{parseStatus.content}</span>
      </div>
      {renderSatisfiesLine(satisfiesStatus)}
    </div>
  );
}

function renderSatisfiesLine(status: StatusResult) {
  if (status.content === null) {
    return (
      <div class="hidden" aria-hidden="true">
        <span>1.0.0 satisfies *</span>
      </div>
    );
  }

  return (
    <div class={getStatusClassName(status.tone)}>
      <span>{status.content}</span>
    </div>
  );
}

function getStatusClassName(tone: StatusResult["tone"]): string {
  const base = "leading-relaxed";
  return tone === "bad" ? `${base} text-red-700` : base;
}
