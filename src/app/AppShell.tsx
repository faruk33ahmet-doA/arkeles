import { ZoomEngine } from "@/navigation/zoom/ZoomEngine";
import { Dock } from "@/navigation/dock/Dock";
import { ZoomTrail } from "@/navigation/trail/ZoomTrail";
import { CommandPalette } from "@/command/CommandPalette";
import { ConflictPanel } from "@/ui/ConflictPanel";
import { useCommandPaletteHotkey } from "@/command/useCommandPalette";
import { useVaultStatus } from "@/data/hooks/useVaultStatus";
import { useVaultChanged } from "@/data/hooks/useVaultChanged";
import { VaultPicker } from "@/modules/onboarding/VaultPicker";
import { useGlobalKeybindings } from "./keybindings";
import { WindowDragRegion } from "./WindowDragRegion";

/*
 * AppShell — Anayasa madde 22.7: "bütün modüller aynı evren içindedir".
 *
 * Kabuk HİÇ unmount olmaz. Dock, Trail ve Palet katman geçişlerinden
 * etkilenmez; yalnız içerideki katman değişir.
 *
 * 17.3  Vault yapılandırılmamışsa tek soru sorulur (VaultPicker).
 * 21.2  Bu bir karşılama ekranı DEĞİL: seçimden sonra bir daha görünmez.
 * 21.3  Durum yüklenirken boş iskelet gösterilmez — kabuk doğrudan gelir.
 */

export function AppShell() {
  useCommandPaletteHotkey();
  useGlobalKeybindings();
  // Madde 20.4 → 21.3: Obsidian'da değişen not, kullanıcı dokunmadan yansır.
  useVaultChanged();

  const { data: vault, isLoading } = useVaultStatus();

  // Vault yolu YOKSA tek soru. `isLoading` sırasında karar verilmez —
  // yanıp sönen bir seçim ekranı göstermek kötü olurdu (madde 5).
  const needsVault = !isLoading && !vault?.path;

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-surface-0">
      {needsVault ? (
        <>
          <WindowDragRegion />
          <VaultPicker />
        </>
      ) : (
        <>
          <ZoomEngine />
          {/* Sürükleme şeridi z-drag'de: trail ve dock'un altında. */}
          <WindowDragRegion />
          <ZoomTrail />
          <Dock />
          <CommandPalette />
          {/* Madde 25.4: akışı kesmeye yetkili TEK durum. */}
          <ConflictPanel />
        </>
      )}
    </div>
  );
}
