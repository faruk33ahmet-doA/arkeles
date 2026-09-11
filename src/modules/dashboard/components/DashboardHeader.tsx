import { useCommandStore } from "@/state/commandStore";

const dateFormatter = new Intl.DateTimeFormat("tr-TR", {
  weekday: "long",
  day: "numeric",
  month: "long",
});

export function DashboardHeader() {
  const openCommandPalette = useCommandStore((state) => state.setOpen);
  const today = dateFormatter.format(new Date());

  return (
    <header className="mb-6 flex items-end justify-between gap-6 border-b border-border-subtle pb-4">
      <div className="flex items-end gap-6">
        <span className="mb-1 text-sm font-semibold tracking-[0.08em] text-accent">
          ARKELÉS
        </span>
        <div>
          <p className="mb-1 text-sm capitalize text-text-tertiary">{today}</p>
          <h1 className="text-2xl font-semibold tracking-[-0.03em] text-text-primary">
            Bugün
          </h1>
        </div>
      </div>

      <button
        type="button"
        onClick={() => openCommandPalette(true)}
        className="flex items-center gap-6 rounded-md border border-border-default bg-dashboard-panel px-4 py-3 text-sm text-text-secondary outline-none transition-colors duration-fast ease-out hover:border-dashboard-rule hover:bg-dashboard-raised hover:text-text-primary focus-visible:ring-1 focus-visible:ring-accent"
      >
        <span>Ara veya yakala</span>
        <kbd className="font-sans text-xs text-text-tertiary">⌘K</kbd>
      </button>
    </header>
  );
}
