/*!
Uygulama yapılandırması — Anayasa madde 17.

17.1  Pencere konumu, son katman, dock sırası, tema, index → VAULT'A YAZILMAZ.
17.2  Bunlar uygulama yapılandırmasıdır, bilgi değildir. app-config dizinine gider.
17.3  Vault yolu HİÇBİR YERDE SABİT KODLANMAZ. İlk açılışta sorulur.

Sprint 0: yapılandırma okunur/oluşturulur ama vault yolu boştur.
          "İlk açılışta sor" akışı Sprint 1'de gelir.
*/

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::{CoreError, CoreResult};

/// Diskte tutulan yapılandırma. Anayasa madde 17.2.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Anayasa madde 17.3: sabit kodlanamaz. Yapılandırılmadıysa `None`.
    #[serde(default)]
    pub vault_path: Option<PathBuf>,
}

impl Config {
    /// Yapılandırmayı app-config dizininden okur; yoksa varsayılanı döner.
    ///
    /// Eksik dosya bir HATA DEĞİLDİR — ilk açılışta normaldir.
    pub fn load(app: &AppHandle) -> CoreResult<Self> {
        let path = Self::file_path(app)?;

        match std::fs::read_to_string(&path) {
            Ok(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(CoreError::Config(err)),
        }
    }

    fn file_path(app: &AppHandle) -> CoreResult<PathBuf> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|_| CoreError::AppDirUnavailable)?;
        Ok(dir.join("config.json"))
    }

    /// Vault yolu — yapılandırılmadıysa açık hata.
    /// Anayasa madde 18.3: yapılandırılmamış olmak çökme sebebi değildir,
    /// arayüz bunu sakin bir durum olarak gösterir.
    pub fn require_vault_path(&self) -> CoreResult<&PathBuf> {
        self.vault_path.as_ref().ok_or(CoreError::VaultNotConfigured)
    }
}
