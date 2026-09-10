import { useMemo } from "react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/ui/primitives/command";
import { Dialog, DialogContent, DialogOverlay, DialogPortal, DialogTitle } from "@/ui/primitives/dialog";
import { useCommandStore } from "@/state/commandStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import { useHermesHealth } from "@/data/hooks/useHermesHealth";
import { getActions, type Action, type ActionContext } from "./actionRegistry";
import { cn } from "@/lib/cn";

/*
 * Komut Paleti — Anayasa madde 27.
 *
 * 27.3  actionRegistry'yi okur. Kayıtta olmayan aksiyon yoktur.
 * 18.2  Hermes'in bildirmediği yetenek BURADA HİÇ GÖRÜNMEZ — filtre altta.
 * 30.1  Cam yüzey İZİNLİ (küçük, sınırlı yüzey). Dock ile birlikte
 *       ekrandaki en fazla 2 cam yüzeyin ikincisi (madde 30.3).
 * 30.2  Cam ASLA animate edilmez: Dialog içeriğine giriş/çıkış animasyonu
 *       verilmez. Ya vardır ya yoktur.
 * 23.2  Bu yüzden burada hiçbir motion bileşeni yok.
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

  const ctx: ActionContext = useMemo(
    () => ({ enterLayer, closePalette: () => setOpen(false) }),
    [enterLayer, setOpen],
  );

  // Anayasa madde 18.2 filtresi: yeteneği olmayan semantik aksiyon gizlenir.
  const visibleActions = useMemo(() => {
    return getActions().filter((action) => {
      if (action.kind !== "semantic") return true;
      if (!action.capability) return false;
      return hermes?.reachable === true && hermes.capabilities.includes(action.capability);
    });
  }, [hermes]);

  const groups = useMemo(() => groupActions(visibleActions), [visibleActions]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
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
          {/* Erişilebilirlik için zorunlu; görsel olarak gizli. */}
          <DialogTitle className="sr-only">Komut paleti</DialogTitle>

          <Command
            // Kendi filtremizi değil cmdk'nın fuzzy eşleşmesini kullanıyoruz.
            className="[&_[cmdk-input-wrapper]]:border-b [&_[cmdk-input-wrapper]]:border-border-subtle"
          >
            <CommandInput
              placeholder="Ara veya git…"
              className={cn(
                "w-full bg-transparent px-4 py-3 text-base text-text-primary",
                "outline-none placeholder:text-text-tertiary",
              )}
            />
            <CommandList className="max-h-[320px] overflow-y-auto p-2">
              {/* Madde 26.2: soğuk "sonuç bulunamadı" değil. */}
              <CommandEmpty className="px-2 py-6 text-sm text-text-tertiary">
                Eşleşen bir şey yok.
              </CommandEmpty>

              {[...groups.entries()].map(([groupName, actions]) => (
                <CommandGroup
                  key={groupName}
                  heading={groupName}
                  className={cn(
                    "[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-2",
                    "[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium",
                    "[&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-[0.08em]",
                    "[&_[cmdk-group-heading]]:text-text-tertiary",
                  )}
                >
                  {actions.map((action) => (
                    <CommandItem
                      key={action.id}
                      value={`${action.title} ${action.keywords?.join(" ") ?? ""}`}
                      onSelect={() => action.run?.(ctx)}
                      className={cn(
                        "cursor-default rounded-sm px-2 py-2 text-base text-text-secondary",
                        "data-[selected=true]:bg-accent-muted data-[selected=true]:text-text-primary",
                      )}
                    >
                      {action.title}
                    </CommandItem>
                  ))}
                </CommandGroup>
              ))}
            </CommandList>
          </Command>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}
