export function SiteFooter() {
  const currentYear = new Date().getFullYear();

  return (
    <footer class="pt-6 font-mono text-sm leading-relaxed text-neutral-500">
      <nav
        class="flex flex-wrap items-center gap-x-4 gap-y-2"
        aria-label="Project links"
      >
        <a
          class="underline underline-offset-4 hover:text-neutral-900"
          href="https://github.com/ryuapp/js-semver"
        >
          GitHub
        </a>
        <a
          class="underline underline-offset-4 hover:text-neutral-900"
          href="https://crates.io/crates/js-semver"
        >
          crates.io
        </a>
        <a
          class="underline underline-offset-4 hover:text-neutral-900"
          href="https://lib.rs/crates/js-semver"
        >
          lib.rs
        </a>
        <a
          class="underline underline-offset-4 hover:text-neutral-900"
          href="https://docs.rs/js-semver"
        >
          docs.rs
        </a>
      </nav>
      <p class="mt-3">
        &copy; {currentYear}{" "}
        <a
          class="text-inherit no-underline hover:text-neutral-900"
          href="https://ryu.app"
        >
          Ryu
        </a>
      </p>
    </footer>
  );
}
