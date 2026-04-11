import type { ComponentChildren } from "preact";

export type CardTone = "good" | "bad" | "neutral";

type ResultCardProps = {
  title: string;
  tone: CardTone;
  label: string;
  detail: ComponentChildren;
  pending?: boolean;
};

export function ResultCard(
  { title, tone, label, detail, pending = false }: ResultCardProps,
) {
  return (
    <article class={`result-card ${tone}`}>
      <header>
        <h2>{title}</h2>
        <span class={`pill ${tone}`}>{label}</span>
      </header>
      <p class={`detail ${pending ? "detail-pending" : ""}`}>
        {detail}
      </p>
    </article>
  );
}
