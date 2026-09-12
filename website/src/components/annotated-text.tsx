import type { ComponentChildren } from "preact";

type AnnotatedTextProps = {
  children: ComponentChildren;
};

export function AnnotatedText({ children }: AnnotatedTextProps) {
  return (
    <>
      <svg class="absolute h-0 w-0" aria-hidden="true">
        <defs>
          <filter
            id="annotated-text-rough"
            x="-30%"
            y="-30%"
            width="160%"
            height="160%"
          >
            <feTurbulence
              type="fractalNoise"
              baseFrequency={0.035}
              numOctaves={2}
              seed={11}
              result="noise"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="noise"
              scale={1.5}
              xChannelSelector="R"
              yChannelSelector="G"
            />
          </filter>
        </defs>
      </svg>
      <span class="relative mr-1 inline-block whitespace-nowrap font-mono text-sm tracking-wider text-neutral-500">
        {children}
        <svg
          aria-hidden="true"
          class="pointer-events-none absolute bottom-[-0.32em] left-[-1%] h-[0.5em] w-[102%] text-neutral-400"
          viewBox="0 0 140 10"
          fill="none"
          preserveAspectRatio="none"
        >
          <path
            d="M3,6 C40,3 100,3 137,5"
            stroke="currentColor"
            strokeWidth={2.4}
            strokeLinecap="round"
            fill="none"
            filter="url(#annotated-text-rough)"
          />
        </svg>
      </span>
    </>
  );
}
