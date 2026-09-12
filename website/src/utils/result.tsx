import type { ComponentChildren } from "preact";
import { Check, X } from "lucide-preact";

export type InputTone = "default" | "good" | "bad";

export type ParseResult =
  | { canonical: string }
  | { error: string }
  | null;

export type SatisfiesResult = {
  rangeCanonical: string;
  versionCanonical: string;
  value: boolean | null;
} | null;

export type StatusResult = {
  tone: "good" | "bad" | "neutral";
  content: ComponentChildren;
};

export function getTrailingVisual(result: ParseResult | null) {
  if (result === null) {
    return null;
  }

  if ("canonical" in result) {
    return <Check aria-hidden="true" size={18} strokeWidth={2.4} />;
  }

  return <X aria-hidden="true" size={18} strokeWidth={2.4} />;
}

export function getInputTone(
  result: ParseResult | null,
): InputTone {
  if (result === null) {
    return "default";
  }

  if ("canonical" in result) {
    return "good";
  }

  return "bad";
}

export function getParseStatus(result: ParseResult | null): StatusResult {
  if (result === null) {
    return {
      tone: "neutral",
      content: "Pending",
    };
  }

  if ("error" in result) {
    return {
      tone: "bad",
      content: `Invalid: ${result.error}`,
    };
  }

  return {
    tone: "good",
    content: (
      <>
        Parseable and normalize to <code>{result.canonical}</code>
      </>
    ),
  };
}

export function getSatisfiesStatus(
  result: SatisfiesResult | null,
): StatusResult {
  if (result === null) {
    return {
      tone: "neutral",
      content: "Pending",
    };
  }

  if (result.value === null) {
    return {
      tone: "neutral",
      content: null,
    };
  }

  return {
    tone: getSatisfiesTone(result.value),
    content: (
      <>
        <code>{result.versionCanonical}</code> {getSatisfiesVerb(result.value)}
        {" "}
        <code>{result.rangeCanonical}</code>
      </>
    ),
  };
}

export function getCompatTone(allMatch: boolean): "good" | "bad" {
  if (allMatch) {
    return "good";
  }

  return "bad";
}

export function getCompatLabel(allMatch: boolean): string {
  if (allMatch) {
    return "Compatible";
  }

  return "Incompatible";
}

function getSatisfiesTone(satisfies: boolean): StatusResult["tone"] {
  if (satisfies) {
    return "good";
  }

  return "bad";
}

function getSatisfiesVerb(satisfies: boolean): string {
  if (satisfies) {
    return "satisfies";
  }

  return "does not satisfy";
}
