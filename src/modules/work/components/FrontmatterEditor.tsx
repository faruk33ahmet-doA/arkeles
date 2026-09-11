import { useMemo, useState } from "react";
import { useFieldMutation } from "@/data/hooks/useNoteMutation";
import { cn } from "@/lib/cn";

/*
 * Frontmatter kontrolü — Anayasa madde 8.1. Sprint 2
 * `set_frontmatter_field` motorunun İLK UI bağlantısı.
 *
 * KAPSAM: yalnız DİZGE alanlar düzenlenebilir. Liste ve nesne alanlar
 * okunur ama düzenlenmez — bir YAML listesini serbest metin kutusundan
 * düzenletmek veri bozma riski taşır ve madde 6.3 uyarınca şüphede
 * eklemiyoruz. Etiketler kendi kontrolünde (TagEditor).
 *
 * Korunan alanlar GİZLENİR: `arkeles_id` (madde 16.4 — Hermes'in işi),
 * `tags` (kendi kontrolü var).
 */

const HIDDEN_KEYS = new Set(["arkeles_id", "tags"]);

interface Field {
  key: string;
  value: string;
  editable: boolean;
}

function readFields(frontmatter: string): Field[] {
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(frontmatter) as Record<string, unknown>;
  } catch {
    return [];
  }

  return Object.entries(parsed)
    .filter(([key]) => !HIDDEN_KEYS.has(key))
    .map(([key, value]) => {
      if (typeof value === "string") return { key, value, editable: true };
      if (typeof value === "number" || typeof value === "boolean") {
        return { key, value: String(value), editable: false };
      }
      return { key, value: Array.isArray(value) ? value.join(", ") : "…", editable: false };
    })
    .sort((a, b) => a.key.localeCompare(b.key, "tr"));
}

interface FrontmatterEditorProps {
  noteId: string;
  frontmatter: string;
  managed: boolean;
}

export function FrontmatterEditor({
  noteId,
  frontmatter,
  managed,
}: FrontmatterEditorProps) {
  const fields = useMemo(() => readFields(frontmatter), [frontmatter]);
  const mutation = useFieldMutation();
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");

  if (fields.length === 0) {
    return <p className="text-sm text-text-tertiary">Frontmatter alanı yok.</p>;
  }

  const commit = (key: string) => {
    const value = draft.trim();
    setEditing(null);
    const current = fields.find((f) => f.key === key)?.value ?? "";
    // Değişiklik yoksa dosyaya dokunmayız (madde 35.1).
    if (value === current) return;
    mutation.mutate({
      noteId,
      key,
      value,
      label: `"${key}" alanı güncelleniyordu.`,
    });
  };

  return (
    <dl className="flex flex-col gap-2">
      {fields.map((field) => (
        <div key={field.key} className="flex items-baseline gap-4">
          <dt className="w-28 shrink-0 truncate text-xs text-text-tertiary">
            {field.key}
          </dt>
          <dd className="min-w-0 flex-1">
            {editing === field.key ? (
              <input
                autoFocus
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                onBlur={() => commit(field.key)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    commit(field.key);
                  }
                  if (event.key === "Escape") setEditing(null);
                }}
                className={cn(
                  "w-full select-text rounded-sm border border-border-strong",
                  "bg-transparent px-2 py-1 text-sm text-text-primary outline-none",
                )}
              />
            ) : (
              <button
                type="button"
                disabled={!managed || !field.editable}
                onClick={() => {
                  setEditing(field.key);
                  setDraft(field.value);
                }}
                className={cn(
                  "w-full truncate rounded-sm px-2 py-1 text-left text-sm",
                  "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
                  managed && field.editable
                    ? "cursor-text text-text-primary hover:bg-surface-3"
                    : "cursor-default text-text-secondary",
                )}
              >
                {field.value || "—"}
              </button>
            )}
          </dd>
        </div>
      ))}
    </dl>
  );
}
