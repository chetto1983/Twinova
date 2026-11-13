import { beforeAll, describe, expect, it } from "vitest";

// Monaco's own Structured Text (spec 2026-11-12-st-editor-design.md, decision 2, amended): the token types it gives the
// constructs of Twinova's sources, so a Monaco upgrade that changes them shows here. Importing Monaco takes about 3 s
// under happy-dom (measured), hence the hook's timeout.
type Tokenize = (text: string, language: string) => { offset: number; type: string }[][];
let tokenize: Tokenize;

beforeAll(async () => {
  const monaco = await import("monaco-editor/editor/editor.api.js");
  const st = await import("monaco-editor/languages/definitions/st/st.js");
  monaco.languages.register({ id: "st" });
  monaco.languages.setMonarchTokensProvider("st", st.language);
  tokenize = monaco.editor.tokenize;
}, 60_000);

/** Each word of `line` with its token type, white space left out. */
function tokens(line: string): string[] {
  const [found] = tokenize(line, "st");
  return found
    .map((t, i) => [line.slice(t.offset, found[i + 1]?.offset ?? line.length), t.type] as const)
    .filter(([text]) => text.trim() !== "")
    .map(([text, type]) => `${text} ${type}`);
}

describe("Monaco's Structured Text", () => {
  it("colours declarations, addresses and types", () => {
    expect(tokens("VAR")).toEqual(["VAR variable.st"]);
    expect(tokens("  sensor AT %IX0.0 : BOOL;")).toEqual([
      "sensor identifier.st",
      "AT keyword.st",
      "%IX0.0 tag.st",
      ": ",
      "BOOL type.st",
      "; delimiter.st",
    ]);
    // Only the * of a partial address stays plain (measured 2026-11-12).
    expect(tokens("  run AT %I* : BOOL;").slice(1, 4)).toEqual(["AT keyword.st", "%I tag.st", "* "]);
  });

  it("colours comments, constants and literals", () => {
    expect(tokens("(* a comment *) jamTimer(IN := TRUE, PT := T#2S); // a line")).toEqual([
      "(* a comment *) comment.st",
      "jamTimer identifier.st",
      "( delimiter.parenthesis.st",
      "IN identifier.st",
      ":= ",
      "TRUE constant.st",
      ", ",
      "PT identifier.st",
      ":= ",
      "T#2S tag.st",
      ") delimiter.parenthesis.st",
      "; delimiter.st",
      "// a line comment.st",
    ]);
    expect(tokens("IF count MOD 3 = 0 THEN mask := 16#FF; END_IF;")).toContain("16#FF number.hex.st");
    expect(tokens("IF a THEN b := 1; END_IF;").filter((t) => t.endsWith("keyword.st"))).toEqual([
      "IF keyword.st",
      "THEN keyword.st",
      "END_IF keyword.st",
    ]);
  });
});
