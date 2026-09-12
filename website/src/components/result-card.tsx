import type { ComponentChildren } from "preact";
import { Check, CircleAlert, LoaderCircle } from "lucide-preact";

export type CardTone = "good" | "bad" | "neutral";

type ResultCardProps = {
  title?: string;
  tone: CardTone;
  label?: string;
  detail?: ComponentChildren;
  className?: string;
  icon?: ComponentChildren;
  pending?: boolean;
  labelMotion?: "from-icon" | "into-icon";
};

export function ResultCard(
  {
    title,
    tone,
    label,
    detail,
    className,
    icon,
    pending = false,
    labelMotion,
  }: ResultCardProps,
) {
  const StatusIcon = getStatusIcon(tone, pending);
  const resultCardClassName = getResultCardClassName(className);
  const pillIconClassName = getPillIconClassName(pending);
  const detailClassName = getDetailClassName(pending);

  return (
    <div class={resultCardClassName}>
      <header class="flex flex-wrap items-center justify-between gap-2.5">
        {renderTitle(title, icon)}
        {label && (
          <span class={getPillClassName(tone, pending)}>
            {StatusIcon && (
              <StatusIcon
                aria-hidden="true"
                class={pillIconClassName}
                size={14}
                strokeWidth={2}
              />
            )}
            <span class={getLabelClassName(labelMotion)}>{label}</span>
          </span>
        )}
      </header>
      {detail && <p class={detailClassName}>{detail}</p>}
    </div>
  );
}

function getLabelClassName(
  motion?: "from-icon" | "into-icon",
): string | undefined {
  if (motion === "into-icon") {
    return "inline-block origin-left overflow-hidden whitespace-nowrap animate-[status-into-icon_160ms_ease-in_forwards]";
  }

  if (motion === "from-icon") {
    return "inline-block overflow-hidden whitespace-nowrap animate-[status-from-icon_200ms_ease-out_both]";
  }

  return undefined;
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

function getResultCardClassName(className?: string): string {
  const base = "py-4";
  return className ? `${base} ${className}` : base;
}

function getPillClassName(tone: CardTone, pending: boolean): string {
  const gap = pending ? "gap-0" : "gap-1.5";
  const base = `inline-flex items-center ${gap} font-mono text-sm`;
  switch (tone) {
    case "good":
      return `${base} text-emerald-700`;
    case "bad":
      return `${base} text-red-700`;
    default:
      return `${base} text-neutral-500`;
  }
}

function renderTitle(title?: string, icon?: ComponentChildren) {
  if (!title) {
    return <span />;
  }

  return (
    <div class="inline-flex items-center gap-2">
      {icon && <span class="inline-flex text-neutral-500">{icon}</span>}
      <h2 class="text-sm font-medium">{title}</h2>
    </div>
  );
}

function getPillIconClassName(pending: boolean): string {
  if (pending) {
    return "inline-flex animate-spin";
  }

  return "inline-flex";
}

function getDetailClassName(pending: boolean): string {
  if (pending) {
    return "mt-2 whitespace-pre-line text-sm leading-relaxed text-neutral-500";
  }

  return "mt-2 whitespace-pre-line text-sm leading-relaxed text-neutral-600";
}
