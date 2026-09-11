import { useState } from "react";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { useNoteDetail } from "@/data/hooks/useWork";
import { useWorkStore } from "@/state/workStore";
import { MovableTaskList } from "./components/MovableTaskList";
import { NoteBody } from "./components/NoteBody";
import { TagEditor } from "./components/TagEditor";
import { FrontmatterEditor } from "./components/FrontmatterEditor";
import { openDocument } from "@/services/work.service";
import { cn } from "@/lib/cn";
import type { NoteLink } from "@/lib/generated";

/*
 * Not detayı — Sprint 3 madde 10, 11.
 *
 * "ARKELÉS metin editörü değildir. Not gövdesi düzenlenemez."
 * Gövde okunur biçimde render edilir; yalnız MEKANİK kontroller
 * (frontmatter alanları, etiketler, görev checkbox'ları) yazılabilir.
 *
 * ZOOM DEĞİL: bu bir alt yüzeydir (madde 22.6). Zoom Trail'e seviye
 * eklemez — Sprint 3 madde 12.
 */

export function NoteDetailView({ noteId }: { noteId: string }) {
  const { data, isLoading, isError } = useNoteDetail(noteId);
  const closeNote = useWorkStore((s) => s.closeNote);

  if (isError) {
    return (
      <EmptyState
        message="Bu not artık index'te yok."
        hint="Obsidian'da silinmiş veya taşınmış olabilir."
      />
    );
  }
  if (!data) return isLoading ? null : null;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h2 className="truncate text-lg font-semibold text-text-primary">
            {data.title}
          </h2>
          <p className="mt-1 select-text truncate text-xs text-text-tertiary">
            {data.sourcePath}
          </p>
        </div>
        <button
          type="button"
          onClick={closeNote}
          className={cn(
            "shrink-0 rounded-sm border border-border-default px-3 py-1 text-sm",
            "text-text-secondary transition-colors duration-fast ease-out",
            "outline-none hover:bg-surface-3 hover:text-text-primary",
            "focus-visible:ring-1 focus-visible:ring-border-strong",
          )}
        >
          Kapat
        </button>
      </div>

      {/* Madde 16.3: yönetilmeyen not sadece okunur — sebebi görünür. */}
      {!data.managed ? (
        <p className="text-sm text-text-tertiary">
          Bu not yönetilmiyor: frontmatter'ında arkeles_id yok, bu yüzden
          değiştirilemez.
        </p>
      ) : null}

      <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
        <div className="flex flex-col gap-6 md:col-span-2">
          <Card title="İçerik">
            <NoteBody body={data.body} />
          </Card>

          {data.tasks.length > 0 ? (
            <Card
              title="Görevler"
              action={
                data.managed && data.tasks.length > 1 ? (
                  <span className="text-xs text-text-tertiary">sürükle · ⌥↑ ⌥↓</span>
                ) : null
              }
            >
              <MovableTaskList tasks={data.tasks} />
            </Card>
          ) : null}
        </div>

        <div className="flex flex-col gap-6">
          <Card title="Alanlar">
            <FrontmatterEditor
              noteId={data.id}
              frontmatter={data.frontmatter}
              managed={data.managed}
            />
          </Card>

          <Card title="Etiketler">
            <TagEditor
              noteId={data.id}
              frontmatter={data.frontmatter}
              managed={data.managed}
            />
          </Card>

          {/* Sprint 3 madde 11: giden ve gelen ayrı. Sprint 5: belgeler ayrı. */}
          <Card title="İlişkili">
            {data.outgoing.length === 0 && data.incoming.length === 0 ? (
              <EmptyState message="Bağlantı yok." />
            ) : (
              <div className="flex flex-col gap-4">
                <LinkGroup
                  label="Giden"
                  links={data.outgoing.filter((link) => !link.documentId)}
                />
                <LinkGroup label="Gelen" links={data.incoming} />
                <DocumentGroup links={data.outgoing.filter((link) => link.documentId)} />
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}

function LinkGroup({ label, links }: { label: string; links: NoteLink[] }) {
  const openNote = useWorkStore((s) => s.openNote);
  if (links.length === 0) return null;

  return (
    <div className="flex flex-col gap-2">
      <span className="text-xs uppercase tracking-[0.08em] text-text-tertiary">
        {label}
      </span>
      <ul className="flex flex-col gap-1">
        {links.map((link, index) => (
          <li key={`${link.title}-${index}`}>
            <button
              type="button"
              disabled={!link.noteId}
              onClick={() => link.noteId && openNote(link.noteId)}
              className={cn(
                "w-full truncate rounded-sm px-1 py-1 text-left text-sm",
                "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
                link.noteId
                  ? "text-text-primary hover:bg-surface-3"
                  : // Kırık bağlantı bir BİLGİDİR; gizlemek veri saklamaktır.
                    "cursor-default text-text-tertiary",
              )}
            >
              {link.title}
              {!link.noteId ? " · yok" : ""}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

/*
 * Belge bağlantıları — Sprint 5 madde 7.
 *
 * Belge bir NOT DEĞİLDİR: not detayında açılmaz, işletim sisteminin kendi
 * uygulamasında açılır. Yol arayüzde YOK; yalnız index kimliği gider ve
 * çekirdek yolu index'ten çözer (madde 19 — güvenli çözücü).
 */
function DocumentGroup({ links }: { links: NoteLink[] }) {
  const [failed, setFailed] = useState(false);
  if (links.length === 0) return null;

  return (
    <div className="flex flex-col gap-2">
      <span className="text-xs uppercase tracking-[0.08em] text-text-tertiary">
        Belgeler
      </span>
      <ul className="flex flex-col gap-1">
        {links.map((link, index) => (
          <li key={`${link.documentId}-${index}`}>
            <button
              type="button"
              onClick={() => {
                setFailed(false);
                if (link.documentId) {
                  openDocument(link.documentId).catch(() => setFailed(true));
                }
              }}
              className={cn(
                "w-full truncate rounded-sm px-1 py-1 text-left text-sm text-text-primary",
                "outline-none hover:bg-surface-3 focus-visible:ring-1 focus-visible:ring-border-strong",
              )}
            >
              {link.title}
            </button>
          </li>
        ))}
      </ul>
      {failed ? <p className="text-xs text-status-attention">Belge açılamadı.</p> : null}
    </div>
  );
}
