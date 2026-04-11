import { useEffect, useMemo, useState } from "preact/hooks";
import semver from "semver";

import { Input } from "./input.tsx";
import { buildCompatIssueUrl } from "./issues.ts";
import {
  getDefaultRange,
  getDefaultVersion,
  readInputsFromQuery,
  writeInputsToQuery,
} from "./query.ts";
import { type CardTone, ResultCard } from "./result-card.tsx";
import init, {
  parse_range,
  parse_version,
  satisfies as satisfies_range,
} from "../wasm/pkg/js_semver_website_wasm.js";

type ParseResult = {
  input: string;
  ok: boolean;
  canonical: string | null;
  error: string | null;
};

type SatisfiesResult = {
  range: ParseResult;
  version: ParseResult;
  satisfies: boolean | null;
};

const COPYRIGHT_YEAR = new Date().getFullYear();

function parseJson<T>(value: string): T {
  return JSON.parse(value) as T;
}

function parseCardState(
  title: string,
  result: ParseResult | null,
): {
  title: string;
  tone: CardTone;
  label: string;
  detail: string;
  pending: boolean;
} {
  if (result === null) {
    return {
      title,
      tone: "neutral",
      label: "Pending",
      detail: "",
      pending: true,
    };
  }

  return result.ok
    ? {
      title,
      tone: "good",
      label: "Valid",
      detail: `Canonical: ${result.canonical}`,
      pending: false,
    }
    : {
      title,
      tone: "bad",
      label: "Invalid",
      detail: result.error ?? "Unknown parse error.",
      pending: false,
    };
}

export function App() {
  const [rangeInput, setRangeInput] = useState(() =>
    readInputsFromQuery().rangeInput
  );
  const [versionInput, setVersionInput] = useState(() =>
    readInputsFromQuery().versionInput
  );
  const [isReady, setIsReady] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    void (async () => {
      try {
        await init();
        if (active) {
          setIsReady(true);
        }
      } catch (error) {
        const message = error instanceof Error
          ? error.message
          : "Failed to initialize wasm";
        if (active) {
          setInitError(message);
        }
      }
    })();

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const onPopState = () => {
      const next = readInputsFromQuery();
      setRangeInput(next.rangeInput);
      setVersionInput(next.versionInput);
    };

    addEventListener("popstate", onPopState);

    return () => {
      removeEventListener("popstate", onPopState);
    };
  }, []);

  const rangeResult = useMemo<ParseResult | null>(() => {
    if (!isReady) {
      return null;
    }
    return parseJson<ParseResult>(parse_range(rangeInput));
  }, [isReady, rangeInput]);

  const versionResult = useMemo<ParseResult | null>(() => {
    if (!isReady) {
      return null;
    }
    return parseJson<ParseResult>(parse_version(versionInput));
  }, [isReady, versionInput]);

  const satisfiesResult = useMemo<SatisfiesResult | null>(() => {
    if (!isReady) {
      return null;
    }
    return parseJson<SatisfiesResult>(
      satisfies_range(rangeInput, versionInput),
    );
  }, [isReady, rangeInput, versionInput]);

  const overallStatus = useMemo(() => {
    if (satisfiesResult === null) {
      return {
        label: "Pending",
        detail: "",
        tone: "neutral" as const,
        pending: true,
      };
    }

    if (satisfiesResult.satisfies === null) {
      return {
        label: "Unavailable",
        detail:
          "`satisfies` is only evaluated after both inputs parse successfully.",
        tone: "neutral" as const,
        pending: false,
      };
    }

    return satisfiesResult.satisfies
      ? {
        label: "Satisfied",
        detail:
          `${satisfiesResult.version.canonical} satisfies ${satisfiesResult.range.canonical}`,
        tone: "good" as const,
        pending: false,
      }
      : {
        label: "Not satisfied",
        detail:
          `${satisfiesResult.version.canonical} does not satisfy ${satisfiesResult.range.canonical}`,
        tone: "bad" as const,
        pending: false,
      };
  }, [satisfiesResult]);

  const compatStatus = useMemo(() => {
    if (
      rangeResult === null || versionResult === null || satisfiesResult === null
    ) {
      return {
        tone: "neutral" as const,
        label: "Pending",
        detail:
          "Range parse: pending\nVersion parse: pending\nSatisfies: pending",
      };
    }

    const nodeRangeOk = semver.validRange(rangeInput) !== null;
    const nodeVersionOk = semver.valid(versionInput) !== null;
    const nodeSatisfies = nodeRangeOk && nodeVersionOk
      ? semver.satisfies(versionInput, rangeInput)
      : null;

    const rangeMatches = rangeResult.ok === nodeRangeOk;
    const versionMatches = versionResult.ok === nodeVersionOk;
    const satisfiesMatches = satisfiesResult.satisfies === nodeSatisfies;
    const allMatch = rangeMatches && versionMatches && satisfiesMatches;
    const issueUrl = buildCompatIssueUrl(rangeInput, versionInput);

    return {
      tone: allMatch ? "good" as const : "bad" as const,
      label: allMatch ? "Compatible" : "Mismatch",
      detail: allMatch
        ? "This js-semver follows node-semver parsing and range semantics."
        : (
          <>
            Compatibility issues were found. You can report them easily using
            the following link:
            <br />
            <a class="inline-link" href={issueUrl}>GitHub Issues</a>
          </>
        ),
    };
  }, [rangeInput, rangeResult, satisfiesResult, versionInput, versionResult]);

  const rangeCard = parseCardState("Range Parse", rangeResult);
  const versionCard = parseCardState("Version Parse", versionResult);

  const handleRangeInputChange = (value: string) => {
    setRangeInput(value);
    writeInputsToQuery(value, versionInput);
  };

  const handleVersionInputChange = (value: string) => {
    setVersionInput(value);
    writeInputsToQuery(rangeInput, value);
  };

  return (
    <main class="page-shell">
      <section class="hero">
        <h1>js-semver playground</h1>
        <p class="hero-copy">
          A parser and evaluator for npm&apos;s flavor of Semantic Versioning.
        </p>
        <p class="hero-link-row">
          <a
            class="hero-link"
            href="https://github.com/ryuapp/js-semver"
          >
            GitHub
          </a>
        </p>
      </section>

      <section class="panel">
        <div class="grid">
          <Input
            label="Range"
            value={rangeInput}
            onValueChange={handleRangeInputChange}
            placeholder={getDefaultRange()}
            tone={rangeResult === null
              ? "default"
              : rangeResult.ok
              ? "good"
              : "bad"}
          />

          <Input
            label="Version"
            value={versionInput}
            onValueChange={handleVersionInputChange}
            placeholder={getDefaultVersion()}
            tone={versionResult === null
              ? "default"
              : versionResult.ok
              ? "good"
              : "bad"}
          />
        </div>

        {initError && (
          <div class="banner bad">WASM init failed: {initError}</div>
        )}

        <div class="result-grid">
          <ResultCard
            title={rangeCard.title}
            tone={rangeCard.tone}
            label={rangeCard.label}
            detail={rangeCard.detail || "Canonical: "}
            pending={rangeCard.pending}
          />

          <ResultCard
            title={versionCard.title}
            tone={versionCard.tone}
            label={versionCard.label}
            detail={versionCard.detail || "Canonical: "}
            pending={versionCard.pending}
          />
        </div>

        <div class="result-stack">
          <ResultCard
            title="Satisfies"
            tone={overallStatus.tone}
            label={overallStatus.label}
            detail={overallStatus.detail || "Result pending."}
            pending={overallStatus.pending}
          />
        </div>

        <div class="result-stack">
          <ResultCard
            title="node-semver compat"
            tone={compatStatus.tone}
            label={compatStatus.label}
            detail={compatStatus.detail}
          />
        </div>
      </section>
      <footer class="page-footer">
        © {COPYRIGHT_YEAR}{" "}
        <a
          class="page-footer-link"
          href="https://ryu.app"
        >
          Ryu
        </a>
      </footer>
    </main>
  );
}
