// A CSS colour as Monaco's themes take it. A built stylesheet's minifier shortens `#ffffff` to `#fff`, and Monaco
// refuses a short hex as a token colour ("Illegal value for token color: #fff", measured 2026-11-14 in the light
// theme, where the editor then never started): a short hex is written out in full.
export function fullHex(color: string): string {
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])([0-9a-f])?$/i.exec(color.trim());
  if (!short) return color.trim();
  return (
    "#" +
    short
      .slice(1)
      .filter(Boolean)
      .map((d) => d + d)
      .join("")
  );
}
