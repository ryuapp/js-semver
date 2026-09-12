import { assertEquals } from "@std/assert";

import {
  initJsSemver,
  parseRange,
  parseVersion,
  satisfies,
} from "./js-semver.ts";

let initPromise: Promise<unknown> | null = null;

async function ensureInit() {
  if (initPromise === null) {
    initPromise = initJsSemver();
  }

  await initPromise;
}

Deno.test("parseRange returns canonical range", async () => {
  await ensureInit();

  const result = parseRange("vvvv1");
  assertEquals(result, ">=1.0.0 <2.0.0-0");
});

Deno.test("parseVersion returns canonical version", async () => {
  await ensureInit();

  const result = parseVersion("1.2.3");
  assertEquals(result, "1.2.3");
});

Deno.test("parseRange throws on invalid input", async () => {
  await ensureInit();

  let message = "";
  try {
    parseRange("v");
  } catch (error) {
    if (error instanceof Error) {
      message = error.message;
    }
  }

  assertEquals(message.length > 0, true);
});

Deno.test("satisfies returns true for matching inputs", async () => {
  await ensureInit();

  const result = satisfies("^1.2.3", "1.3.0");
  assertEquals(result, true);
});

Deno.test("satisfies returns null when parsing fails", async () => {
  await ensureInit();

  const result = satisfies("v", "1.3.0");
  assertEquals(result, null);
});
