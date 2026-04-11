import { useId, useState } from "preact/hooks";

type FieldTone = "default" | "good" | "bad";

type BaseFieldProps = {
  label: string;
  value: string;
  placeholder?: string;
  tone?: FieldTone;
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

  return (
    <label
      className="field"
      for={id}
      data-filled={filled ? "" : undefined}
      data-focused={focused ? "" : undefined}
      data-tone={toneClassName}
    >
      <span className="field-label">{props.label}</span>
      {props.multiline
        ? (
          <textarea
            {...textareaProps}
            rows={props.rows ?? 3}
            onInput={(event) => props.onValueChange(event.currentTarget.value)}
          />
        )
        : (
          <input
            {...inputProps}
            onInput={(event) => props.onValueChange(event.currentTarget.value)}
          />
        )}
    </label>
  );
}
