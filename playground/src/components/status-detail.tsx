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
    <div class="status-list">
      <div class={`status-line ${status.tone}`}>
        <span>{status.content}</span>
      </div>
      <div class="status-line status-line-spacer" aria-hidden="true">
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
    <div class="status-list">
      <div class={`status-line ${parseStatus.tone}`}>
        <span>{parseStatus.content}</span>
      </div>
      {renderSatisfiesLine(satisfiesStatus)}
    </div>
  );
}

function renderSatisfiesLine(status: StatusResult) {
  if (status.content === null) {
    return (
      <div class="status-line status-line-spacer" aria-hidden="true">
        <span>1.0.0 satisfies *</span>
      </div>
    );
  }

  return (
    <div class={`status-line ${status.tone}`}>
      <span>{status.content}</span>
    </div>
  );
}
