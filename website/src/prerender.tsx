import { renderToString } from "preact-render-to-string";

import { NotFound } from "./404.tsx";
import { App } from "./app.tsx";
import { getDefaultRange, getDefaultVersion } from "./utils/query.ts";

export function prerender(data: { url: string }) {
  const isNotFoundPage = data.url === "/404.html";

  return {
    html: renderToString(
      isNotFoundPage ? <NotFound /> : (
        <App
          initialRangeInput={getDefaultRange()}
          initialVersionInput={getDefaultVersion()}
        />
      ),
    ),
    head: {
      title: isNotFoundPage
        ? "Not Found"
        : "js-semver - semver crate compliant with node-semver",
      elements: isNotFoundPage
        ? new Set([
          <meta key="robots" name="robots" content="noindex" />,
        ])
        : new Set(),
    },
  };
}
