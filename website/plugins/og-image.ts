import { render } from "takumi-js";
import type { Plugin } from "vite";

async function renderOgImage(): Promise<Uint8Array> {
  return await render(
    `
      <div style="width: 100%; height: 100%; display: flex; flex-direction: column; justify-content: space-between; padding: 72px; background: #ffffff; color: #171717; font-family: sans-serif;">
        <div style="font-size: 32px; font-weight: 700; letter-spacing: -0.04em;">js-semver</div>
        <div style="display: flex; flex-direction: column; gap: 24px;">
          <div style="font-size: 58px; font-weight: 700; line-height: 1.08; letter-spacing: -0.045em;">Parser and evaluator for npm's flavor of Semantic Versioning, compliant with node-semver.</div>
          <div style="font-size: 28px; line-height: 1.4; color: #525252;">It maintains high compatibility and performance, and has zero dependencies by default.</div>
        </div>
      </div>
    `,
    { width: 1200, height: 630 },
  );
}

async function writeOgImage(outputPath: string): Promise<void> {
  const resolvedOutputPath = new URL(`../${outputPath}`, import.meta.url);

  await Deno.mkdir(new URL("./", resolvedOutputPath), { recursive: true });
  const image = await renderOgImage();
  await Deno.writeFile(resolvedOutputPath, image);
}

export default function ogImage(): Plugin {
  return {
    name: "render-og-image",
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        const url = new URL(request.url ?? "/", "http://localhost");
        if (url.pathname !== "/og.png") {
          next();
          return;
        }

        try {
          const image = await renderOgImage();
          response.statusCode = 200;
          response.setHeader("Content-Type", "image/png");
          response.setHeader("Content-Length", image.byteLength);
          response.setHeader("Cache-Control", "no-store");
          response.end(image);
        } catch (error) {
          next(error);
        }
      });
    },
    async closeBundle() {
      await writeOgImage("dist/og.png");
    },
  };
}
