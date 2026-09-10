/*!
Uygulama yapılandırması — Anayasa madde 17.

17.1  Pencere konumu, son katman, dock sırası, tema, index → VAULT'A YAZILMAZ.
17.2  Bunlar uygulama yapılandırmasıdır, bilgi değildir. app-config dizinine gider.
17.3  Vault yolu HİÇBİR YERDE SABİT KODLANMAZ. İlk açılışta sorulur.
*/

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::{CoreError, CoreResult};

/// Diskte tutulan yapılandırma. Anayasa madde 17.2.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigData {
    /// Anayasa madde 17.3: sabit kodlanamaz. Yapılandırılmadıysa `None`.
    #[serde(default)]
    pub vault_path: Option<PathBuf>,

    /*
     * Hızlı Yakalama gelen kutusu — Anayasa madde 10.2.
     *
     * Vault köküne göre YOL. ARKELÉS bu dosyayı OLUŞTURMAZ (madde 10.2
     * "yeni dosya açmaz"); yalnız var olana ham satır ekler. Dosya yoksa
     * Hızlı Yakalama arayüzde görünmez (madde 18.2).
     */
    #[serde(default)]
    pub inbox_path: Option<String>,
}

/// Gelen kutusu için varsayılan yol. Kullanıcı veya Hermes bu dosyayı
/// oluşturana kadar Hızlı Yakalama pasif kalır.
pub const DEFAULT_INBOX: &str = "Gelen Kutusu.md";

/// Yapılandırmanın canlı hali. Vault değiştiğinde yazılabilir olmalı,
/// bu yüzden RwLock: okuma çok, yazma nadir.
pub struct Config {
    data: RwLock<ConfigData>,
    file: PathBuf,
}

impl Config {
    /// Yapılandırmayı app-config dizininden okur; yoksa varsayılanı döner.
    /// Eksik dosya bir HATA DEĞİLDİR — ilk açılışta normaldir.
    pub fn load(app: &AppHandle) -> CoreResult<Self> {
        let file = Self::file_path(app)?;

        let data = match std::fs::read_to_string(&file) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => ConfigData::default(),
            Err(err) => return Err(CoreError::Config(err)),
        };

        Ok(Self {
            data: RwLock::new(data),
            file,
        })
    }

    fn file_path(app: &AppHandle) -> CoreResult<PathBuf> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|_| CoreError::AppDirUnavailable)?;
        Ok(dir.join("config.json"))
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, ConfigData> {
        self.data.read().unwrap_or_else(|p| p.into_inner())
    }

    /// Yapılandırılmış vault yolu. Madde 17.3: `None` geçerli bir durumdur.
    pub fn vault_path(&self) -> Option<PathBuf> {
        self.read().vault_path.clone()
    }

    /// Gelen kutusu yolu — Anayasa madde 10.2.
    pub fn inbox_path(&self) -> String {
        self.read()
            .inbox_path
            .clone()
            .unwrap_or_else(|| DEFAULT_INBOX.to_string())
    }

    /// Vault yolunu değiştirir ve diske yazar.
    ///
    /// Yol okunabilir bir dizin değilse REDDEDİLİR — yapılandırmayı bozmayız.
    pub fn set_vault_path(&self, path: &Path) -> CoreResult<()> {
        if !crate::vault::reader::is_readable(path) {
            return Err(CoreError::VaultUnreadable);
        }

        {
            let mut data = self.data.write().unwrap_or_else(|p| p.into_inner());
            data.vault_path = Some(path.to_path_buf());
        }

        self.persist()
    }

    fn persist(&self) -> CoreResult<()> {
        if let Some(parent) = self.file.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::Config)?;
        }

        let json = serde_json::to_string_pretty(&*self.read())
            .map_err(|e| CoreError::Config(std::io::Error::other(e)))?;

        // Madde 20.1 ile aynı disiplin: geçici dosya + rename.
        // Yapılandırma vault değil ama yarım yazılmış config de kabul edilemez.
        let tmp = self.file.with_extension("json.tmp");
        std::fs::write(&tmp, json).map_err(CoreError::Config)?;
        std::fs::rename(&tmp, &self.file).map_err(CoreError::Config)?;

        Ok(())
    }
}
