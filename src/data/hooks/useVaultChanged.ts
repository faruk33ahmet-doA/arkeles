import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "@/services/ipc";
import { VAULT_DEPENDENT_KEYS, queryKeys } from "@/data/queryKeys";

/*
 * Vault değişim aboneliği — Anayasa madde 20.4, 21.3.
 *
 * 21.3: "Kullanıcı boş bir iskelet ekranı beklemez."
 * Obsidian'da bir not değişince çekirdek index'i günceller ve
 * `vault:changed` yayınlar; burada o event TanStack Query invalidation'ına
 * çevrilir. Kullanıcı ARKELÉS'e DOKUNMADAN güncel veriyi görür.
 *
 * Yoklama (polling) YOK — madde 35.1: gereksiz iş yasak.
 */
export function useVaultChanged(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listen("vault:changed", () => {
      for (const key of VAULT_DEPENDENT_KEYS) {
        queryClient.invalidateQueries({ queryKey: key });
      }
      // Tarama bitmiş olabilir: index durumu da tazelenir.
      queryClient.invalidateQueries({ queryKey: queryKeys.index.status });
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [queryClient]);
}
