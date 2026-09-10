import { useQuery } from "@tanstack/react-query";
import { getVaultStatus } from "@/services/vault.service";
import { queryKeys } from "@/data/queryKeys";

/** Anayasa madde 17.3: vault yolu yapılandırılmamış olabilir — null geçerli sonuç. */
export function useVaultStatus() {
  return useQuery({
    queryKey: queryKeys.vault.status,
    queryFn: getVaultStatus,
    retry: false,
  });
}
