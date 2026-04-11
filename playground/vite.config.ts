import preact from "@preact/preset-vite";
import { defineConfig } from "vite";
import { vitePrerenderPlugin } from "vite-prerender-plugin";
import ogImage from "./plugins/og-image.ts";

export default defineConfig({
  build: {
    assetsDir: "_next/static",
  },
  plugins: [
    preact(),
    vitePrerenderPlugin({
      renderTarget: "#root",
      prerenderScript: new URL("./src/prerender.tsx", import.meta.url).href,
      additionalPrerenderRoutes: ["/404.html"],
      previewMiddlewareFallback: "/404.html",
    }),
    ogImage(),
  ],
});
