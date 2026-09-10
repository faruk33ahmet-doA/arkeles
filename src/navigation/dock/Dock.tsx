import { useNavigationStore } from "@/state/navigationStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import { getDockItems } from "./dockItems";
import { cn } from "@/lib/cn";

/*
 * Bottom Dock — Anayasa madde 22.1 (sidebar YOK), 22.2 (dock VAR).
 *
 * Cam yüzey (madde 30): dock küçük ve sınırlı bir yüzeydir → cam İZİNLİ.
 * Asla animate edilmez (madde 30.2). Ekranda aynı anda en fazla 2 cam
 * yüzeyden biri (diğeri: Cmd+K).
 *
 * Sprint 0: dock görünür ve tıklanınca katman değiştirir. Görsel cila
 * (ikonlar, hover mikro-hareket) referans tasarım geldikten sonra (madde 28.1).
 * İkon YOK çünkü emoji yasak (madde 32) ve özel ikon seti henüz yok —
 * şimdilik metin etiketi, sessiz ve dürüst.
 */

export function Dock() {
  const items = getDockItems();
  const activeLayer = useNavigationStore((s) => s.trail[s.trail.length - 1]);
  const { enterLayer } = useZoom();

  return (
    <nav
      aria-label="Modüller"
      className={cn(
        "pointer-events-auto fixed inset-x-0 bottom-0 z-dock flex justify-center pb-4",
      )}
    >
      <div
        className={cn(
          "flex items-center gap-1 rounded-lg border border-glass-border px-2 py-2",
          "bg-glass-bg shadow-2 backdrop-blur-glass",
        )}
      >
        {items.map((item) => {
          const isActive = item.id === activeLayer;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => enterLayer(item.id)}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "rounded-sm px-3 py-2 text-sm font-medium transition-colors duration-fast ease-out",
                "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
                isActive
                  ? "bg-accent-muted text-text-primary"
                  : "text-text-secondary hover:text-text-primary",
                item.status === "placeholder" && !isActive && "text-text-tertiary",
              )}
            >
              {item.title}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
