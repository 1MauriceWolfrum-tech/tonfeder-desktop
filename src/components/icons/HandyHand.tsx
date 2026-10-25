// Tonfeder-Zeichen als einfarbiges Symbol (Text-Cursor aus Pegelbalken).
// Der Komponentenname bleibt, damit Updates aus Handy nicht kollidieren.
const HandyHand = ({
  width,
  height,
}: {
  width?: number | string;
  height?: number | string;
}) => (
  <svg
    width={width || 126}
    height={height || 135}
    viewBox="14 6 36 52"
    className="fill-text"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="18" y="10" width="28" height="5" />
    <rect x="18" y="49" width="28" height="5" />
    <rect x="18.2" y="27" width="3.6" height="10" />
    <rect x="24.2" y="22" width="3.6" height="20" />
    <rect x="30.2" y="17" width="3.6" height="30" />
    <rect x="36.2" y="21" width="3.6" height="22" />
    <rect x="42.2" y="26" width="3.6" height="12" />
  </svg>
);

export default HandyHand;
