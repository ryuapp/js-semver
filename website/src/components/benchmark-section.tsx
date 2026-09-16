import benchmarkData from "../../benchmarks/results.json" with { type: "json" };

type BenchmarkRow = (typeof benchmarkData.native.rows)[number];

const numberFormatter = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

function formatDuration(nanoseconds: number) {
  if (nanoseconds >= 10_000) {
    return `${numberFormatter.format(nanoseconds / 1_000)} µs`;
  }
  return `${numberFormatter.format(nanoseconds)} ns`;
}

function BenchmarkBars({ rows }: { rows: BenchmarkRow[] }) {
  return (
    <div class="border-t border-neutral-200">
      {rows.map((row) => {
        const maximum = Math.max(row.jsSemverNs, row.nodeSemverNs);
        const jsSemverWidth = Math.max((row.jsSemverNs / maximum) * 100, 2);
        const nodeSemverWidth = Math.max(
          (row.nodeSemverNs / maximum) * 100,
          2,
        );

        return (
          <div class="border-b border-neutral-200 py-4" key={row.label}>
            <h3 class="break-words font-mono text-sm text-neutral-700">
              {row.label}
            </h3>
            <div class="mt-3 space-y-2.5 font-mono text-sm tabular-nums">
              <div class="grid grid-cols-[6.5rem_minmax(0,1fr)_6.5rem] items-center gap-2">
                <span class="font-semibold text-neutral-950">js-semver</span>
                <div class="h-2.5 overflow-hidden bg-neutral-100">
                  <div
                    class="h-full bg-neutral-900"
                    style={{ width: `${jsSemverWidth}%` }}
                  />
                </div>
                <span class="text-right font-semibold text-neutral-950">
                  {formatDuration(row.jsSemverNs)}
                </span>
              </div>
              <div class="grid grid-cols-[6.5rem_minmax(0,1fr)_6.5rem] items-center gap-2">
                <span class="text-neutral-500">node-semver</span>
                <div class="h-2.5 overflow-hidden bg-neutral-100">
                  <div
                    class="h-full bg-neutral-300"
                    style={{ width: `${nodeSemverWidth}%` }}
                  />
                </div>
                <span class="text-right text-neutral-500">
                  {formatDuration(row.nodeSemverNs)}
                </span>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function BenchmarkSection() {
  return (
    <section class="mt-7" id="benchmarks" aria-labelledby="benchmarks-heading">
      <h2
        class="text-2xl font-semibold tracking-tight"
        id="benchmarks-heading"
      >
        Benchmarks
      </h2>
      <p class="mt-2 text-sm leading-relaxed text-neutral-600">
        A comparison between the node-semver crate and js-semver.
      </p>

      <div class="mt-6">
        <p class="text-sm leading-relaxed text-neutral-500">
          <a
            class="underline underline-offset-4 hover:text-neutral-900"
            href="https://crates.io/crates/js-semver"
          >
            js-semver {benchmarkData.native.jsSemverVersion}
          </a>{" "}
          vs{" "}
          <a
            class="underline underline-offset-4 hover:text-neutral-900"
            href="https://crates.io/crates/node-semver"
          >
            node-semver crate {benchmarkData.native.nodeSemverVersion}
          </a>
        </p>
        <div class="mt-3">
          <BenchmarkBars rows={benchmarkData.native.rows} />
        </div>
        <p class="mt-2 font-mono text-sm leading-relaxed text-neutral-500">
          Last updated: {benchmarkData.generatedAt.slice(0, 10)} · Source:{" "}
          <a
            class="underline underline-offset-4 hover:text-neutral-900"
            href="https://github.com/ryuapp/js-semver/blob/main/website/benchmarks/benches/comparison.rs"
          >
            comparison.rs
          </a>
        </p>
      </div>
    </section>
  );
}
