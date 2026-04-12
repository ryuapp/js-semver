import type { ComponentChildren } from "preact";
import { useId, useState } from "preact/hooks";

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

function getToneClassName(tone: FieldTone): string {
  switch (tone) {
    case "good":
      return "good";
    case "bad":
      return "bad";
    default:
      return "default";
  }
}

export function Input(props: FieldProps) {
  const id = useId();
  const [focused, setFocused] = useState(false);
  const filled = props.value.trim().length > 0;
  const toneClassName = getToneClassName(props.tone ?? "default");

  const sharedProps = {
    id,
    className: "field-control",
    onFocus: () => setFocused(true),
    onBlur: () => setFocused(false),
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
  const fieldDataFilled = getFieldDataAttribute(filled);
  const fieldDataFocused = getFieldDataAttribute(focused);
  const fieldBody = renderFieldBody(props, inputProps, textareaProps);

  return (
    <label
      className="field"
      for={id}
      data-filled={fieldDataFilled}
      data-focused={fieldDataFocused}
      data-tone={toneClassName}
    >
      <span className="field-label">{props.label}</span>
      {fieldBody}
      <div className="field-detail">{props.detail}</div>
    </label>
  );
}

function getFieldDataAttribute(enabled: boolean): "" | undefined {
  if (enabled) {
    return "";
  }

  return undefined;
}

function renderFieldBody(
  props: FieldProps,
  inputProps: {
    id: string;
    className: string;
    onFocus: () => void;
    onBlur: () => void;
    placeholder: string | undefined;
    value: string;
  },
  textareaProps: {
    id: string;
    className: string;
    onFocus: () => void;
    onBlur: () => void;
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
    <span className="field-input-wrap">
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
    <span className="field-trailing-visual">
      {trailingVisual}
    </span>
  );
}
