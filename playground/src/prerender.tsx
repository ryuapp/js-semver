import { renderToString } from "preact-render-to-string";

function StaticApp() {
  const currentYear = new Date().getFullYear();

  return (
    <main class="page-shell">
      <section class="hero">
        <h1>js-semver playground</h1>
        <p class="hero-copy">
          A parser and evaluator for npm&apos;s flavor of Semantic Versioning.
        </p>
        <p class="hero-link-row">
          <a class="hero-link" href="https://github.com/ryuapp/js-semver">
            GitHub
          </a>
        </p>
      </section>

      <section class="panel">
        <div class="grid">
          <label class="field" data-tone="default">
            <span class="field-label">Range</span>
            <input
              class="field-control"
              value="^1.2.3"
              placeholder=">=1.2.3 <2.0.0-0"
              readOnly
            />
          </label>

          <label class="field" data-tone="default">
            <span class="field-label">Version</span>
            <input
              class="field-control"
              value="1.5.0"
              placeholder="1.5.0"
              readOnly
            />
          </label>
        </div>

        <div class="result-grid">
          <article class="result-card neutral">
            <header>
              <h2>Range Parse</h2>
              <span class="pill neutral">Pending</span>
            </header>
            <p class="detail">Canonical:</p>
          </article>

          <article class="result-card neutral">
            <header>
              <h2>Version Parse</h2>
              <span class="pill neutral">Pending</span>
            </header>
            <p class="detail">Canonical:</p>
          </article>
        </div>

        <article class="result-card neutral">
          <header>
            <h2>Satisfies</h2>
            <span class="pill neutral">Pending</span>
          </header>
          <p class="detail">Result pending.</p>
        </article>
      </section>

      <footer class="page-footer">
        © {currentYear}{" "}
        <a class="page-footer-link" href="https://ryu.app">Ryu</a>
      </footer>
    </main>
  );
}

function StaticNotFound() {
  const currentYear = new Date().getFullYear();

  return (
    <main class="page-shell">
      <section class="hero">
        <h1>404</h1>
        <p class="hero-copy">
          The page you requested could not be found.
        </p>
        <p class="hero-link-row">
          <a class="hero-link" href="/">
            Back to playground
          </a>
        </p>
      </section>

      <footer class="page-footer">
        © {currentYear}{" "}
        <a class="page-footer-link" href="https://ryu.app">Ryu</a>
      </footer>
    </main>
  );
}

export function prerender(data: { url: string }) {
  const isNotFoundPage = data.url === "/404.html";

  return {
    html: isNotFoundPage
      ? renderToString(<StaticNotFound />)
      : renderToString(<StaticApp />),
    head: {
      title: isNotFoundPage ? "Not Found" : "js-semver playground",
      elements: isNotFoundPage
        ? new Set([
          <meta key="robots" name="robots" content="noindex" />,
        ])
        : new Set(),
    },
  };
}
