import { render } from "preact";

import { NotFound } from "./404.tsx";
import { App } from "./app.tsx";
import "./styles.css";

const root = document.getElementById("root");

if (root === null) {
  throw new Error("Missing #root element");
}

const pathname = globalThis.location.pathname;
const isPlaygroundPath = pathname === "/" || pathname === "/index.html";

document.title = isPlaygroundPath ? "js-semver playground" : "Not Found";

render(isPlaygroundPath ? <App /> : <NotFound />, root);
