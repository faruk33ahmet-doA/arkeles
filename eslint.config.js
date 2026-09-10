import js from "@eslint/js";
import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import reactHooks from "eslint-plugin-react-hooks";

/*
 * Anayasa madde 35.1 ("gereksiz render yasaktır") ve 39.1 ("teknik borç
 * oluşturulmaz") kurallarının otomatik uygulanabilen kısmı.
 *
 * react-hooks kuralları HATA seviyesinde: bağımlılık dizisi hatası,
 * gereksiz render'ın en yaygın kaynağıdır ve sessizce birikir.
 */
export default [
  { ignores: ["dist/**", "src-tauri/**", "node_modules/**"] },
  js.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaVersion: 2022, sourceType: "module", ecmaFeatures: { jsx: true } },
      globals: { window: "readonly", document: "readonly", KeyboardEvent: "readonly", HTMLDivElement: "readonly", React: "readonly" },
    },
    plugins: { "@typescript-eslint": tsPlugin, "react-hooks": reactHooks },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",
      "no-undef": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "@typescript-eslint/consistent-type-imports": "error",
    },
  },
];
