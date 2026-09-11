import { useMemo, useState } from "react";
import { useTagMutation } from "@/data/hooks/useNoteMutation";
import { cn } from "@/lib/cn";

/*
 * Etiket kontrolü — Anayasa madde 8.1. Sprint 2 `toggle_tag` motorunun
 * İLK UI bağlantısı.
 *
 * Madde 16.3: yönetilmeyen notta kontroller GÖRÜNMEZ — çalışmayan bir
 * alan göstermek madde 18.2 ihlali olurdu.
 */

function readTags(frontmatter: string): string[] {
  try {
    const parsed = JSON.parse(frontmatter) as Record<string, unknown>;
    const raw = parsed.tags;
    if (Array.isArray(raw)) return raw.filter((t): t is string => typeof t === "string");
    if (typeof raw === "string") return [raw];
  } catch {
    // Bozuk frontmatter etiket göstermeyi engellemez, sadece boş kalır.
  }
  return [];
}

interface TagEditorProps {
  noteId: string;
  frontmatter: string;
  managed: boolean;
}

export function TagEditor({ noteId, frontmatter, managed }: TagEditorProps) {
  const tags = useMemo(() => readTags(frontmatter), [frontmatter]);
  const [draft, setDraft] = useState("");
  const mutation = useTagMutation();

  const add = () => {
    const tag = draft.trim();
    if (!tag) return;
    mutation.mutate({ noteId, tag, add: true });
    setDraft("");
  };

  return (
    <div className="flex flex-col gap-3">
      {tags.length > 0 ? (
        <div className="flex flex-wrap gap-2">
          {tags.map((tag) => (
            <span
              key={tag}
              className={cn(
                "inline-flex items-center gap-2 rounded-sm border border-border-subtle",
                "bg-surface-3 px-2 py-1 text-xs text-text-secondary",
              )}
            >
              {tag}
              {managed ? (
                <button
                  type="button"
                  aria-label={`${tag} etiketini kaldır`}
                  onClick={() => mutation.mutate({ noteId, tag, add: false })}
                  className={cn(
                    "text-text-tertiary transition-colors duration-fast ease-out",
                    "outline-none hover:text-text-primary",
                    "focus-visible:ring-1 focus-visible:ring-border-strong",
                  )}
                >
                  ×
                </button>
              ) : null}
            </span>
          ))}
        </div>
      ) : (
        <p className="text-sm text-text-tertiary">Etiket yok.</p>
      )}

      {/* Madde 16.3 + 18.2: yazılamayan notta giriş alanı gösterilmez. */}
      {managed ? (
        <div className="flex items-center gap-2">
          <input
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                add();
              }
            }}
            placeholder="Etiket ekle"
            className={cn(
              "min-w-0 flex-1 select-text rounded-sm border border-border-subtle",
              "bg-transparent px-2 py-1 text-sm text-text-primary outline-none",
              "placeholder:text-text-tertiary focus-visible:border-border-strong",
            )}
          />
          <button
            type="button"
            onClick={add}
            disabled={!draft.trim() || mutation.isPending}
            className={cn(
              "shrink-0 rounded-sm border border-border-default px-2 py-1 text-sm",
              "text-text-secondary transition-colors duration-fast ease-out",
              "outline-none hover:text-text-primary hover:bg-surface-3",
              "focus-visible:ring-1 focus-visible:ring-border-strong",
              "disabled:cursor-default disabled:text-text-tertiary",
            )}
          >
            Ekle
          </button>
        </div>
      ) : null}
    </div>
  );
}
