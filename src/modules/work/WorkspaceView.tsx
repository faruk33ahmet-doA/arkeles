import { useWorkStore, WORK_PANELS, type WorkPanel } from "@/state/workStore";
import { OverviewPanel } from "./panels/OverviewPanel";
import { TasksPanel } from "./panels/TasksPanel";
import { ProjectsPanel } from "./panels/ProjectsPanel";
import { DocumentsPanel } from "./panels/DocumentsPanel";
import { MeetingsPanel } from "./panels/MeetingsPanel";
import { NotesPanel } from "./panels/NotesPanel";
import { NoteDetailView } from "./NoteDetailView";
import { cn } from "@/lib/cn";

/*
 * Çalışma alanı şablonu — Sprint 3 madde 3.
 *
 * TEK, YENİDEN KULLANILABİLİR YAPI. WIF ve GEN bu bileşeni AYNI şekilde
 * kullanır; aralarındaki tek fark `workspaceId`. Kuruma özel dal YOK.
 *
 * Madde 22.6 + Sprint 3 madde 4: panel geçişleri ZOOM DEĞİL. Dekoratif
 * animasyon yok — panel anında değişir. Bu, madde 23.2'nin ("dekoratif
 * hareket yasak") doğrudan uygulanmasıdır.
 *
 * Madde 15 (Sprint 3): SIDEBAR YOK. Panel gezinmesi yatay sekme şeridi.
 */

type PanelComponent = (props: { workspaceId: string }) => React.ReactNode;

const PANELS: Record<WorkPanel, PanelComponent> = {
  overview: OverviewPanel,
  tasks: TasksPanel,
  projects: ProjectsPanel,
  documents: DocumentsPanel,
  meetings: MeetingsPanel,
  notes: NotesPanel,
};

export function WorkspaceView({ workspaceId }: { workspaceId: string }) {
  const panel = useWorkStore((s) => s.panel);
  const setPanel = useWorkStore((s) => s.setPanel);
  const noteId = useWorkStore((s) => s.noteId);

  const Panel = PANELS[panel];

  return (
    <div className="flex flex-col gap-6">
      {/* Yatay sekme şeridi — sol menü DEĞİL (Sprint 3 madde 15). */}
      <nav
        aria-label="Çalışma alanı bölümleri"
        className="flex flex-wrap items-center gap-1 border-b border-border-subtle pb-3"
      >
        {WORK_PANELS.map((item) => {
          const isActive = item.id === panel && !noteId;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setPanel(item.id)}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "rounded-sm px-3 py-1 text-sm font-medium",
                "transition-colors duration-fast ease-out",
                "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
                isActive
                  ? "bg-accent-muted text-text-primary"
                  : "text-text-secondary hover:text-text-primary",
              )}
            >
              {item.label}
            </button>
          );
        })}
      </nav>

      {/*
        Not detayı panelin ÜSTÜNE gelir, onun yerine geçmez: kullanıcı
        "Kapat" deyince aynı panele döner. Zoom Trail'e seviye EKLENMEZ
        (Sprint 3 madde 12).
      */}
      {noteId ? <NoteDetailView noteId={noteId} /> : <Panel workspaceId={workspaceId} />}
    </div>
  );
}
