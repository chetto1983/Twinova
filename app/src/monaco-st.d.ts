// Monaco ships its Structured Text language without types (monaco-editor 0.57.0, esm/vs/languages/definitions/st).
declare module "monaco-editor/languages/definitions/st/st.js" {
  import type { languages } from "monaco-editor/editor/editor.api.js";
  export const conf: languages.LanguageConfiguration;
  export const language: languages.IMonarchLanguage;
}
