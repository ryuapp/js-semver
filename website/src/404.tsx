import { SiteFooter } from "./components/site-footer.tsx";

export function NotFound() {
  return (
    <main class="mx-auto min-h-svh max-w-xl px-4 pb-8 text-neutral-900 md:px-0 md:pb-14">
      <section>
        <h1 class="text-5xl font-bold tracking-tight">404</h1>
        <p class="mt-5 text-base leading-snug">
          The page you requested could not be found.
        </p>
        <p class="mt-2">
          <a
            class="text-sm font-semibold underline underline-offset-4"
            href="/"
          >
            Back to playground
          </a>
        </p>
      </section>

      <SiteFooter />
    </main>
  );
}
