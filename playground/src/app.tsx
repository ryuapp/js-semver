import { useEffect, useMemo, useState } from "preact/hooks";

import { CompatCard } from "./components/compat-card.tsx";
import { Input } from "./components/input.tsx";
import {
  RangeStatusDetail,
  VersionStatusDetail,
} from "./components/status-detail.tsx";
import { getCommitHash, getCommitLink } from "./utils/commit.ts";
import {
  getInitErrorMessage,
  initJsSemver,
  parseRange,
  parseVersion,
  satisfies,
} from "./utils/js-semver.ts";
import {
  getDefaultRange,
  getDefaultVersion,
  readInputsFromQuery,
  writeInputsToQuery,
} from "./utils/query.ts";
import {
  getInputTone,
  getTrailingVisual,
  type ParseResult,
  type SatisfiesResult,
} from "./utils/result.tsx";

const COPYRIGHT_YEAR = new Date().getFullYear();
const COMMIT_HASH_FULL = import.meta.env.VITE_COMMIT_HASH;
const COMMIT_HASH = getCommitHash(COMMIT_HASH_FULL);

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
        await initJsSemver();
        if (active) {
          setIsReady(true);
        }
      } catch (error) {
        const message = getInitErrorMessage(error);
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
    try {
      return { canonical: parseRange(rangeInput) };
    } catch (error) {
      return { error: getInitErrorMessage(error) };
    }
  }, [isReady, rangeInput]);

  const versionResult = useMemo<ParseResult | null>(() => {
    if (!isReady) {
      return null;
    }
    try {
      return { canonical: parseVersion(versionInput) };
    } catch (error) {
      return { error: getInitErrorMessage(error) };
    }
  }, [isReady, versionInput]);

  const satisfiesResult = useMemo<SatisfiesResult | null>(() => {
    if (!isReady) {
      return null;
    }

    if (rangeResult === null || versionResult === null) {
      return null;
    }

    if ("error" in rangeResult || "error" in versionResult) {
      return {
        rangeCanonical: "",
        versionCanonical: "",
        value: null,
      };
    }

    return {
      rangeCanonical: rangeResult.canonical,
      versionCanonical: versionResult.canonical,
      value: satisfies(rangeInput, versionInput),
    };
  }, [isReady, rangeInput, rangeResult, versionInput, versionResult]);

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
            tone={getInputTone(rangeResult)}
            trailingVisual={getTrailingVisual(rangeResult)}
            detail={<RangeStatusDetail result={rangeResult} />}
          />

          <Input
            label="Version"
            value={versionInput}
            onValueChange={handleVersionInputChange}
            placeholder={getDefaultVersion()}
            tone={getInputTone(versionResult)}
            trailingVisual={getTrailingVisual(versionResult)}
            detail={
              <VersionStatusDetail
                parseResult={versionResult}
                satisfiesResult={satisfiesResult}
              />
            }
          />
        </div>

        {initError && (
          <div class="banner bad">WASM init failed: {initError}</div>
        )}

        <div class="result-stack">
          <CompatCard
            rangeInput={rangeInput}
            versionInput={versionInput}
            rangeResult={rangeResult}
            versionResult={versionResult}
            satisfiesResult={satisfiesResult}
            commitHash={COMMIT_HASH_FULL}
          />
        </div>

        <div class="panel-meta">
          <a
            class="panel-meta-link"
            href={getCommitLink(COMMIT_HASH_FULL)}
          >
            {COMMIT_HASH}
          </a>
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
