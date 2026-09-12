import { hydrate, render } from "preact";

import { NotFound } from "./404.tsx";
import { App } from "./app.tsx";
import "./styles.css";

const root = document.getElementById("root");

if (root === null) {
  throw new Error("Missing #root element");
}

const pathname = globalThis.location.pathname;
const isWebsitePath = pathname === "/" || pathname === "/index.html";

document.title = isWebsitePath
  ? "js-semver - semver crate compliant with node-semver"
  : "Not Found";

const app = isWebsitePath ? <App /> : <NotFound />;

if (root.hasChildNodes()) {
  hydrate(app, root);
} else {
  render(app, root);
}
