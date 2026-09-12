import type { ComponentChildren } from "preact";
import { useId } from "preact/hooks";

type FieldTone = "default" | "good" | "bad";

type BaseFieldProps = {
  label: string;
  value: string;
  placeholder?: string;
  tone?: FieldTone;
  detail: ComponentChildren;
  trailingVisual?: ComponentChildren;
  onValueChange: (value: string) => void;
};

type InputFieldProps = BaseFieldProps & {
  multiline?: false;
};

type TextareaFieldProps = BaseFieldProps & {
  multiline: true;
  rows?: number;
};

type FieldProps = InputFieldProps | TextareaFieldProps;

export function Input(props: FieldProps) {
  const id = useId();
  const error = props.tone === "bad";
  const detailId = `${id}-detail`;
  const controlClassName = [
    "block h-11 w-full rounded-md border bg-white px-4 pr-11",
    "font-mono text-base text-neutral-900 placeholder:text-neutral-400",
    "outline-none transition-colors disabled:cursor-not-allowed disabled:bg-neutral-100",
    error
      ? "border-red-600 focus:border-red-800"
      : "border-neutral-300 hover:border-neutral-400 focus:border-neutral-900",
  ].join(" ");

  const sharedProps = {
    id,
    className: controlClassName,
    "aria-describedby": detailId,
    "aria-invalid": error,
    spellcheck: false,
    autoComplete: "off",
    autoCapitalize: "off",
  };

  const inputProps = {
    ...sharedProps,
    placeholder: props.placeholder,
    value: props.value,
  };

  const textareaProps = {
    ...sharedProps,
    placeholder: props.placeholder,
    value: props.value,
  };
  const fieldBody = renderFieldBody(props, inputProps, textareaProps);

  return (
    <div
      className="group grid w-full gap-1.5"
      data-tone={props.tone ?? "default"}
    >
      <label
        className="font-mono text-sm font-medium text-neutral-900"
        for={id}
      >
        {props.label}
      </label>
      {fieldBody}
      <div
        className={error
          ? "min-h-5 text-[13px] leading-relaxed text-red-700"
          : "min-h-5 text-[13px] leading-relaxed text-neutral-600"}
        id={detailId}
      >
        {props.detail}
      </div>
    </div>
  );
}

function renderFieldBody(
  props: FieldProps,
  inputProps: {
    id: string;
    className: string;
    placeholder: string | undefined;
    value: string;
  },
  textareaProps: {
    id: string;
    className: string;
    placeholder: string | undefined;
    value: string;
  },
) {
  if (props.multiline) {
    return (
      <textarea
        {...textareaProps}
        rows={getTextareaRows(props.rows)}
        onInput={(event) => props.onValueChange(event.currentTarget.value)}
      />
    );
  }

  return (
    <span className="relative flex min-w-0">
      <input
        {...inputProps}
        onInput={(event) => props.onValueChange(event.currentTarget.value)}
      />
      {renderTrailingVisual(props.trailingVisual)}
    </span>
  );
}

function getTextareaRows(rows?: number): number {
  if (rows !== undefined) {
    return rows;
  }

  return 3;
}

function renderTrailingVisual(trailingVisual?: ComponentChildren) {
  if (!trailingVisual) {
    return null;
  }

  return (
    <span className="pointer-events-none absolute top-1/2 right-3.5 inline-flex -translate-y-1/2 text-emerald-700 group-data-[tone=bad]:text-red-700">
      {trailingVisual}
    </span>
  );
}
