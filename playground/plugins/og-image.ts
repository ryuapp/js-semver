import type { Plugin } from "vite";

async function writeOgImage(outputPath: string): Promise<void> {
  const imageUrl = Deno.env.get("JS_SEMVER_OG_URL");
  const authHeader = Deno.env.get("JS_SEMVER_OG_AUTH");

  if (imageUrl === undefined || imageUrl.length === 0) {
    console.error("JS_SEMVER_OG_URL is not set");
    return;
  }

  const resolvedOutputPath = new URL(`../${outputPath}`, import.meta.url);

  await Deno.mkdir(new URL("./", resolvedOutputPath), { recursive: true });

  let response: Response;
  try {
    response = await fetch(imageUrl, {
      headers: authHeader === undefined ? undefined : {
        Authorization: authHeader,
      },
    });
  } catch (error) {
    console.error("Failed to fetch OGP image:", error);
    return;
  }

  if (!response.ok) {
    console.error(
      `Failed to fetch OGP image: ${response.status} ${response.statusText}`,
    );
    return;
  }

  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.startsWith("image/")) {
    console.error(`JS_SEMVER_OG_URL did not return an image: ${contentType}`);
    return;
  }

  const image = new Uint8Array(await response.arrayBuffer());
  await Deno.writeFile(resolvedOutputPath, image);
}

export default function ogImage(): Plugin {
  return {
    name: "download-og-image",
    async closeBundle() {
      await writeOgImage("dist/og.png");
    },
  };
}
