import React from "react";

// Tonfeder-Logo: Zeichen (Text-Cursor aus Pegelbalken) und kantiger Schriftzug.
// Quelle: design/logo im Repo tonfeder. Der Komponentenname bleibt, damit
// Updates aus Handy nicht mit umbenannten Dateien kollidieren.
const HandyTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="0 0 150 30"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label="Tonfeder"
    >
      <svg x="0" y="1" width="28" height="28" viewBox="0 0 64 64">
        <rect width="64" height="64" rx="3" className="logo-primary" />
        <g fill="#0a0f0b">
          <rect x="18" y="10" width="28" height="5" />
          <rect x="18" y="49" width="28" height="5" />
          <rect x="18.2" y="27" width="3.6" height="10" />
          <rect x="24.2" y="22" width="3.6" height="20" />
          <rect x="30.2" y="17" width="3.6" height="30" />
          <rect x="36.2" y="21" width="3.6" height="22" />
          <rect x="42.2" y="26" width="3.6" height="12" />
        </g>
      </svg>
      <g
        transform="translate(38 0)"
        strokeWidth="3"
        strokeLinecap="square"
        strokeLinejoin="miter"
      >
        <path
          d="M3.5 7.5V22.5H8M1.5 13.5H7.5M14 13.5H23V22.5H14ZM29 22.5V13.5H38V22.5"
          style={{ stroke: "var(--color-logo-primary)" }}
        />
        <path
          d="M46 22.5V7.5H51.5M44 13.5H50M66.5 22.5H57.5V13.5H66.5V18H57.5M81.5 7.5V22.5H72.5V13.5H81.5M96.5 22.5H87.5V13.5H96.5V18H87.5M102.5 22.5V13.5H109.5V16.5"
          style={{ stroke: "var(--color-text)" }}
        />
      </g>
    </svg>
  );
};

export default HandyTextLogo;
