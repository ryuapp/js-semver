import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import { ArrowDown } from "lucide-preact";

import { AnnotatedText } from "./components/annotated-text.tsx";
import { CompatCard } from "./components/compat-card.tsx";
import { GitHubLogo } from "./components/github-logo.tsx";
import { Input } from "./components/input.tsx";
import { SiteFooter } from "./components/site-footer.tsx";
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

const COMMIT_HASH_FULL = import.meta.env.VITE_COMMIT_HASH;
const COMMIT_HASH = getCommitHash(COMMIT_HASH_FULL);
const EXAMPLES = [
  { label: "Caret", range: "^1.2.3", version: "1.5.0" },
  { label: "Tilde", range: "~1.2.3", version: "1.3.0" },
  { label: "Prerelease", range: "^1.0.0-rc.2", version: "1.0.0-rc.3" },
  { label: "Union", range: "^1.0.0 || ^3.0.0", version: "3.2.1" },
];

type AppProps = {
  initialRangeInput?: string;
  initialVersionInput?: string;
};

export function App({ initialRangeInput, initialVersionInput }: AppProps = {}) {
  const [rangeInput, setRangeInput] = useState(() =>
    initialRangeInput ?? readInputsFromQuery().rangeInput
  );
  const [versionInput, setVersionInput] = useState(() =>
    initialVersionInput ?? readInputsFromQuery().versionInput
  );
  const [isCompatLoading, setIsCompatLoading] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);
  const compatTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const startCompatLoading = () => {
    if (compatTimer.current !== null) {
      clearTimeout(compatTimer.current);
    }

    setIsCompatLoading(true);
    compatTimer.current = setTimeout(() => {
      setIsCompatLoading(false);
      compatTimer.current = null;
    }, 400);
  };

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
      startCompatLoading();
      setRangeInput(next.rangeInput);
      setVersionInput(next.versionInput);
    };

    addEventListener("popstate", onPopState);

    return () => {
      removeEventListener("popstate", onPopState);
      if (compatTimer.current !== null) {
        clearTimeout(compatTimer.current);
      }
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
    startCompatLoading();
    setRangeInput(value);
    writeInputsToQuery(value, versionInput);
  };

  const handleVersionInputChange = (value: string) => {
    startCompatLoading();
    setVersionInput(value);
    writeInputsToQuery(rangeInput, value);
  };

  return (
    <main class="mx-auto min-h-svh max-w-xl px-4 pb-8 text-neutral-900 md:px-0 md:pb-14">
      <header class="mb-6 flex items-center justify-between gap-6 py-2">
        <h1 class="text-xl font-bold tracking-tight">js-semver</h1>
        <a
          class="inline-flex items-center gap-1.5 rounded-sm border-[0.5px] border-neutral-200 bg-white px-2 py-1 text-sm font-semibold text-neutral-700 transition-colors hover:border-neutral-400 hover:text-neutral-900"
          href="https://github.com/ryuapp/js-semver"
        >
          <GitHubLogo />
          GitHub
        </a>
      </header>
      <section>
        <p class="m-0 text-base leading-snug">
          <span class="block">
            Parser and evaluator for npm&apos;s flavor of Semantic Versioning,
            compliant with node-semver.
          </span>
          <span class="mt-[1lh] block">
            This crate is designed for the JavaScript ecosystem and follows{" "}
            <a
              class="underline underline-offset-4"
              href="https://github.com/npm/node-semver"
            >
              node-semver
            </a>{" "}
            (the one npm uses) parsing and range semantics. It maintains high
            compatibility and performance, and has zero dependencies by default.
          </span>
        </p>
      </section>

      <div class="mt-7 flex items-baseline justify-between gap-4">
        <h2 class="text-2xl font-semibold tracking-tight">Playground</h2>
        <a
          class="shrink-0 font-mono text-sm text-neutral-500 underline underline-offset-4 hover:text-neutral-900"
          href={getCommitLink(COMMIT_HASH_FULL)}
        >
          {COMMIT_HASH}
        </a>
      </div>
      <div
        class="mt-3 flex flex-wrap items-center gap-2"
        role="group"
        aria-label="Try an example"
      >
        <AnnotatedText>TRY IT</AnnotatedText>
        {EXAMPLES.map((example) => (
          <button
            key={example.label}
            class="min-h-8 rounded-md border border-neutral-300 bg-white px-3 py-1 font-mono text-sm text-neutral-700 transition-colors hover:border-neutral-900 hover:text-neutral-900"
            type="button"
            onClick={() => {
              startCompatLoading();
              setRangeInput(example.range);
              setVersionInput(example.version);
              writeInputsToQuery(example.range, example.version);
            }}
          >
            {example.label}
          </button>
        ))}
      </div>
      <section
        class="mt-5"
        aria-label="Semver playground"
      >
        {initError && (
          <div class="mb-5 text-sm text-red-700">
            WASM init failed: {initError}
          </div>
        )}
        <div class="grid gap-6">
          <div class="min-w-0">
            <Input
              label="Range"
              value={rangeInput}
              onValueChange={handleRangeInputChange}
              placeholder={getDefaultRange()}
              tone={getInputTone(rangeResult)}
              trailingVisual={getTrailingVisual(rangeResult)}
              detail={<RangeStatusDetail result={rangeResult} />}
            />

            <div
              class="my-5 flex items-center gap-2 font-mono text-sm text-neutral-500 after:h-px after:flex-1 after:bg-neutral-200"
              aria-hidden="true"
            >
              <ArrowDown size={16} />
              <span>compare against</span>
            </div>
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
          <div class="flex min-w-0 flex-col">
            <CompatCard
              rangeInput={rangeInput}
              versionInput={versionInput}
              rangeResult={rangeResult}
              versionResult={versionResult}
              satisfiesResult={satisfiesResult}
              loading={isCompatLoading}
              commitHash={COMMIT_HASH_FULL}
            />
          </div>
        </div>
      </section>
      <SiteFooter />
    </main>
  );
}
