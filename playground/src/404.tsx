const COPYRIGHT_YEAR = new Date().getFullYear();

export function NotFound() {
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
        © {COPYRIGHT_YEAR}{" "}
        <a class="page-footer-link" href="https://ryu.app">
          Ryu
        </a>
      </footer>
    </main>
  );
}
