import { useEffect, useState } from "react";
import { useHermesActions, useSubmitJob } from "@/data/hooks/useHermes";
import { useHermesStore } from "@/state/hermesStore";
import { cn } from "@/lib/cn";

/*
 * Semantik aksiyon başlatma — Sprint 4 madde 5, 6.
 *
 * "Yeni devasa AI sohbet ekranı oluşturma. ARKELÉS Hermes arayüzü değildir.
 *  Aksiyon çağırma mümkün olduğunca kısa ve görev odaklı olsun."
 *
 * Bu yüzden: model seçici yok, sistem istemi yok, sohbet geçmişi yok,
 * dosya ekleme yok. Bir aksiyon seçimi ve bir metin alanı.
 *
 * Madde 18.2: Hermes'in bildirmediği aksiyon HİÇ görünmez. Hiçbiri
 * bildirilmemişse bu bileşen HİÇ render edilmez.
 */

interface ActionLauncherProps {
  workspace: string | null;
}

export function ActionLauncher({ workspace }: ActionLauncherProps) {
  const { data: actions } = useHermesActions();
  const submit = useSubmitJob();
  const [openAction, setOpenAction] = useState<string | null>(null);
  const [text, setText] = useState("");

  /*
   * Cmd+K'dan gelen aksiyon — Sprint 4 madde 18.
   *
   * Palet bir aksiyon istediğinde bu yüzey onu açar ve isteği TEMİZLER;
   * böylece aynı istek ikinci bir yüzeyde tekrar açılmaz.
   */
  const pendingAction = useHermesStore((s) => s.pendingAction);
  const pendingWorkspace = useHermesStore((s) => s.pendingWorkspace);
  const clearPending = useHermesStore((s) => s.clear);

  useEffect(() => {
    if (!pendingAction) return;
    // İstek bu yüzeye mi ait? (kurumsuz istek yalnız kurumsuz yüzeye)
    if (pendingWorkspace !== workspace) return;
    setOpenAction(pendingAction);
    clearPending();
  }, [pendingAction, pendingWorkspace, workspace, clearPending]);

  const available = actions ?? [];
  // Madde 18.2: yetenek yoksa yüzey de yok.
  if (available.length === 0) return null;

  const send = () => {
    const input = text.trim();
    if (!input || !openAction || submit.isPending) return;
    submit.mutate({ action: openAction, workspace, input });
    // Madde 6: Hermes cevabı BEKLENMEZ — yüzey hemen kapanır.
    setText("");
    setOpenAction(null);
  };

  if (!openAction) {
    return (
      <div className="flex flex-wrap gap-2">
        {available.map((action) => (
          <button
            key={action.id}
            type="button"
            onClick={() => setOpenAction(action.id)}
            className={cn(
              "rounded-sm border border-border-default px-3 py-2 text-sm font-medium",
              "text-text-secondary transition-colors duration-fast ease-out",
              "outline-none hover:bg-surface-3 hover:text-text-primary",
              "focus-visible:ring-1 focus-visible:ring-border-strong",
            )}
          >
            {action.label}
          </button>
        ))}
      </div>
    );
  }

  const activeLabel = available.find((a) => a.id === openAction)?.label ?? openAction;

  return (
    <div className="flex flex-col gap-3">
      <span className="text-xs uppercase tracking-[0.08em] text-text-tertiary">
        {activeLabel}
      </span>

      <textarea
        autoFocus
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            send();
          }
          if (event.key === "Escape") {
            setOpenAction(null);
            setText("");
          }
        }}
        rows={3}
        placeholder="Ne yapmasını istiyorsun?"
        className={cn(
          "w-full resize-none select-text rounded-sm border border-border-subtle",
          "bg-transparent p-3 text-base text-text-primary outline-none",
          "placeholder:text-text-tertiary focus-visible:border-border-strong",
        )}
      />

      <div className="flex items-center justify-between gap-4">
        <span className="text-xs text-text-tertiary">
          Gönderdiğinde beklemezsin; iş kuyrukta görünür.
        </span>
        <div className="flex shrink-0 gap-2">
          <button
            type="button"
            onClick={() => {
              setOpenAction(null);
              setText("");
            }}
            className={secondaryButton}
          >
            Vazgeç
          </button>
          <button
            type="button"
            onClick={send}
            disabled={!text.trim()}
            className={cn(secondaryButton, "text-text-primary")}
          >
            Gönder
          </button>
        </div>
      </div>
    </div>
  );
}

const secondaryButton = cn(
  "rounded-sm border border-border-default px-3 py-1 text-sm font-medium",
  "text-text-secondary transition-colors duration-fast ease-out",
  "outline-none hover:bg-surface-3 hover:text-text-primary",
  "focus-visible:ring-1 focus-visible:ring-border-strong",
  "disabled:cursor-default disabled:text-text-tertiary",
);
