import { useMemo, useState } from "react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/ui/primitives/command";
import {
  Dialog,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
} from "@/ui/primitives/dialog";
import { useCommandStore } from "@/state/commandStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import { useHermesHealth } from "@/data/hooks/useHermesHealth";
import { useSearch } from "@/data/hooks/useSearch";
import { useInboxStatus } from "@/data/hooks/useInboxStatus";
import { useWorkspaces } from "@/data/hooks/useWork";
import { useHermesActions } from "@/data/hooks/useHermes";
import { useHermesStore } from "@/state/hermesStore";
import { useWorkStore } from "@/state/workStore";
import { QuickCapture } from "./QuickCapture";
import { getActions, type Action, type ActionContext } from "./actionRegistry";
import { cn } from "@/lib/cn";

/*
 * Komut Paleti — Anayasa madde 27.
 *
 * 27.3  actionRegistry'yi okur. Kayıtta olmayan aksiyon yoktur.
 * 27.4  Not arama (FTS5) buradan yapılır — Sprint 1'de bağlandı.
 * 18.2  Hermes'in bildirmediği yetenek BURADA HİÇ GÖRÜNMEZ.
 * 30.1  Cam yüzey İZİNLİ (küçük, sınırlı yüzey) — dock ile birlikte
 *       ekrandaki en fazla 2 cam yüzeyin ikincisi (madde 30.3).
 * 30.2  Cam ASLA animate edilmez → burada hiçbir motion bileşeni yok.
 */

function groupActions(actions: Action[]): Map<string, Action[]> {
  const groups = new Map<string, Action[]>();
  for (const action of actions) {
    const existing = groups.get(action.group);
    if (existing) existing.push(action);
    else groups.set(action.group, [action]);
  }
  return groups;
}

export function CommandPalette() {
  const open = useCommandStore((s) => s.open);
  const setOpen = useCommandStore((s) => s.setOpen);
  const { enterLayer } = useZoom();
  const { data: hermes } = useHermesHealth();

  const [query, setQuery] = useState("");
  const { data: hits } = useSearch(query);
  const { data: inbox } = useInboxStatus();
  const { data: workspaces } = useWorkspaces();
  const { data: hermesAvailable } = useHermesActions();
  const openHermesPanel = useHermesStore((s) => s.open);
  const openWorkspaceInStore = useWorkStore((s) => s.openWorkspace);
  const setPanel = useWorkStore((s) => s.setPanel);

  /*
   * Palet iki kipte çalışır: aksiyon listesi ve Hızlı Yakalama.
   * Kip değişimi ANİDİR — geçiş animasyonu yok (madde 30.2: cam yüzey
   * animate edilmez, ve madde 23.2: dekoratif hareket yasak).
   */
  const [mode, setMode] = useState<"list" | "capture">("list");

  const close = () => {
    setOpen(false);
    setQuery("");
    setMode("list");
  };

  const ctx: ActionContext = useMemo(
    () => ({
      enterLayer,
      closePalette: () => {
        setOpen(false);
        setQuery("");
        setMode("list");
      },
      openQuickCapture: () => setMode("capture"),
      // Sprint 3 madde 13: tek navigasyon yolu — İş katmanına zoom edip
      // çalışma alanını açar. Paralel bir sistem kurulmaz.
      openWorkspace: (workspaceId, panel) => {
        openWorkspaceInStore(workspaceId);
        if (panel) setPanel(panel);
        enterLayer("work");
      },
      // Sprint 4 madde 18: tek navigasyon yolu — kurumu aç, aksiyonu hazırla.
      openHermesAction: (actionId, workspaceId) => {
        if (workspaceId) {
          openWorkspaceInStore(workspaceId);
          setPanel("overview");
          enterLayer("work");
        } else {
          enterLayer("hermes");
        }
        openHermesPanel(actionId, workspaceId);
      },
    }),
    [enterLayer, setOpen, openWorkspaceInStore, setPanel, openHermesPanel],
  );

  // Madde 18.2 filtresi: yeteneği olmayan semantik aksiyon gizlenir.
  const visibleActions = useMemo(() => {
    return getActions({
      inboxAvailable: inbox?.exists === true,
      workspaces,
      hermesActions: hermesAvailable,
    }).filter((action) => {
      if (action.kind !== "semantic") return true;
      if (!action.capability) return false;
      return (
        hermes?.reachable === true && hermes.capabilities.includes(action.capability)
      );
    });
  }, [hermes, inbox, workspaces, hermesAvailable]);

  const groups = useMemo(() => groupActions(visibleActions), [visibleActions]);

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) {
          setQuery("");
          setMode("list");
        }
      }}
    >
      <DialogPortal>
        {/* Overlay: sakin karartma, blur YOK (madde 30.1 — büyük yüzey) */}
        <DialogOverlay className="fixed inset-0 z-command bg-black/40" />
        <DialogContent
          className={cn(
            "fixed left-1/2 top-[20%] z-command w-full max-w-[520px] -translate-x-1/2",
            "overflow-hidden rounded-lg border border-glass-border",
            "bg-glass-bg shadow-3 backdrop-blur-glass-strong",
            "outline-none",
          )}
        >
          <DialogTitle className="sr-only">
            {mode === "capture" ? "Hızlı Yakalama" : "Komut paleti"}
          </DialogTitle>

          {mode === "capture" ? (
            <QuickCapture inboxPath={inbox?.path ?? ""} onDone={close} />
          ) : (
          <Command
            // cmdk'nın kendi filtresi aksiyonlar için; not sonuçları
            // çekirdekten FTS5 ile GELDİĞİ İÇİN yeniden filtrelenmemeli.
            shouldFilter={false}
            className="[&_[cmdk-input-wrapper]]:border-b [&_[cmdk-input-wrapper]]:border-border-subtle"
          >
            <CommandInput
              value={query}
              onValueChange={setQuery}
              placeholder="Ara veya git…"
              className={cn(
                "w-full bg-transparent px-4 py-3 text-base text-text-primary",
                "outline-none placeholder:text-text-tertiary",
              )}
            />
            <CommandList className="max-h-[320px] overflow-y-auto p-2">
              <CommandEmpty className="px-2 py-6 text-sm text-text-tertiary">
                Eşleşen bir şey yok.
              </CommandEmpty>

              {/* Aksiyonlar — sorgu varsa isme göre süzülür. */}
              {[...groups.entries()].map(([groupName, actions]) => {
                const matching = filterActions(actions, query);
                if (matching.length === 0) return null;
                return (
                  <CommandGroup
                    key={groupName}
                    heading={groupName}
                    className={groupHeadingClass}
                  >
                    {matching.map((action) => (
                      <CommandItem
                        key={action.id}
                        value={action.id}
                        onSelect={() => action.run?.(ctx)}
                        className={itemClass}
                      >
                        {action.title}
                      </CommandItem>
                    ))}
                  </CommandGroup>
                );
              })}

              {/*
                Not sonuçları — madde 27.4.
                SPRINT 1 KISITI: seçilince not AÇILMAZ, çünkü not görüntüleme
                katmanı henüz yok. Madde 18.2 ("tıklandığında hata veren buton
                yoktur") gereği sonuçlar `disabled` olarak listelenir: arama
                çalıştığı görülür, yanıltıcı bir eylem sunulmaz.
              */}
              {hits && hits.length > 0 ? (
                <CommandGroup heading="Notlar" className={groupHeadingClass}>
                  {hits.map((hit) => (
                    <CommandItem
                      key={hit.noteId}
                      value={hit.noteId}
                      disabled
                      className={cn(itemClass, "cursor-default")}
                    >
                      <span className="flex min-w-0 flex-col gap-1">
                        <span className="truncate text-text-primary">{hit.title}</span>
                        {hit.snippet ? (
                          <span className="truncate text-xs text-text-tertiary">
                            {hit.snippet}
                          </span>
                        ) : null}
                      </span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              ) : null}
            </CommandList>
          </Command>
          )}
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}

/** Aksiyon süzme — başlık ve anahtar kelimelerde büyük/küçük harf duyarsız. */
function filterActions(actions: Action[], query: string): Action[] {
  const q = query.trim().toLocaleLowerCase("tr");
  if (!q) return actions;
  return actions.filter((action) => {
    const haystack = [action.title, ...(action.keywords ?? [])]
      .join(" ")
      .toLocaleLowerCase("tr");
    return haystack.includes(q);
  });
}

const groupHeadingClass = cn(
  "[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-2",
  "[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium",
  "[&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-[0.08em]",
  "[&_[cmdk-group-heading]]:text-text-tertiary",
);

const itemClass = cn(
  "cursor-default rounded-sm px-2 py-2 text-base text-text-secondary",
  "data-[selected=true]:bg-accent-muted data-[selected=true]:text-text-primary",
  "data-[disabled=true]:opacity-100",
);
