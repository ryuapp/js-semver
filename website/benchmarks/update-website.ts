type BenchEvent = {
  type: "bench";
  name: string;
  median: number;
};

type ResultRow = {
  label: string;
  jsSemverNs: number;
  nodeSemverNs: number;
};

const repositoryRoot = new URL("../../", import.meta.url);

async function runBenchmarks() {
  const command = new Deno.Command("cargo", {
    args: [
      "+nightly",
      "bench",
      "--manifest-path",
      "benchmarks/Cargo.toml",
      "--bench",
      "comparison",
      "--",
      "--format",
      "json",
      "-Z",
      "unstable-options",
    ],
    cwd: new URL("../", import.meta.url),
    stdout: "piped",
  });
  const output = await command.output();
  if (!output.success) {
    throw new Error(`Benchmarks failed with exit code ${output.code}`);
  }

  const measurements = new Map<string, number>();
  for (const line of new TextDecoder().decode(output.stdout).split("\n")) {
    try {
      const event = JSON.parse(line) as BenchEvent;
      if (event.type === "bench") measurements.set(event.name, event.median);
    } catch {
      // Cargo emits non-JSON status lines alongside the benchmark events.
    }
  }
  return measurements;
}

async function readPackageVersion() {
  const manifest = await Deno.readTextFile(
    new URL("Cargo.toml", repositoryRoot),
  );
  const match = /^version = "([^"]+)"$/m.exec(manifest);
  if (match === null) {
    throw new Error("Unable to read the js-semver package version");
  }
  return match[1];
}

const measurements = await runBenchmarks();

function measurement(name: string) {
  const value = measurements.get(name);
  if (value === undefined) throw new Error(`Missing benchmark result: ${name}`);
  return value;
}

const nativeCases = [
  { label: "Parse version", name: "version" },
  { label: "Parse range", name: "range" },
  { label: "Parse + Satisfies", name: "parse_and_satisfies" },
];

const nativeRows: ResultRow[] = nativeCases.map(({ label, name }) => ({
  label,
  jsSemverNs: measurement(`${name}_js_semver`),
  nodeSemverNs: measurement(`${name}_node_semver`),
}));

const result = {
  generatedAt: new Date().toISOString(),
  native: {
    jsSemverVersion: await readPackageVersion(),
    nodeSemverVersion: "2.2.0",
    rows: nativeRows,
  },
};

const destination = new URL("results.json", import.meta.url);
await Deno.writeTextFile(destination, `${JSON.stringify(result, null, 2)}\n`);
