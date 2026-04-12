import type { ComponentChildren } from "preact";
import { Check, CircleAlert, LoaderCircle } from "lucide-preact";

export type CardTone = "good" | "bad" | "neutral";

type ResultCardProps = {
  title?: string;
  tone: CardTone;
  label?: string;
  detail: ComponentChildren;
  className?: string;
  icon?: ComponentChildren;
  pending?: boolean;
};

export function ResultCard(
  { title, tone, label, detail, className, icon, pending = false }:
    ResultCardProps,
) {
  const StatusIcon = getStatusIcon(tone, pending);
  const resultCardClassName = getResultCardClassName(tone, className);
  const pillIconClassName = getPillIconClassName(pending);
  const detailClassName = getDetailClassName(pending);

  return (
    <div class={resultCardClassName}>
      <header>
        {renderTitle(title, icon)}
        {label && (
          <span class={`pill ${tone}`}>
            {StatusIcon && (
              <StatusIcon
                aria-hidden="true"
                class={pillIconClassName}
                size={14}
                strokeWidth={2}
              />
            )}
            {label}
          </span>
        )}
      </header>
      <p class={detailClassName}>{detail}</p>
    </div>
  );
}

function getStatusIcon(tone: CardTone, pending: boolean) {
  if (pending) {
    return LoaderCircle;
  }

  if (tone === "good") {
    return Check;
  }

  if (tone === "bad") {
    return CircleAlert;
  }

  return null;
}

function getResultCardClassName(tone: CardTone, className?: string): string {
  if (className) {
    return `result-card ${tone} ${className}`;
  }

  return `result-card ${tone}`;
}

function renderTitle(title?: string, icon?: ComponentChildren) {
  if (!title) {
    return <span />;
  }

  return (
    <div class="result-card-title">
      {icon && <span class="result-card-icon">{icon}</span>}
      <h2>{title}</h2>
    </div>
  );
}

function getPillIconClassName(pending: boolean): string {
  if (pending) {
    return "pill-icon pill-icon-spin";
  }

  return "pill-icon";
}

function getDetailClassName(pending: boolean): string {
  if (pending) {
    return "detail detail-pending";
  }

  return "detail";
}
