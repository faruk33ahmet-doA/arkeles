/*!
Vault izleyici — Anayasa madde 20.4, 15.6.

20.4  Dosya izleyici event'leri 200 ms penceresinde BİRLEŞTİRİLİR (debounce).
15.6  İçerik hash'i aynıysa yeniden indeksleme yapılmaz.
20.5  Senkronizasyon artefaktları yok sayılır.

NEDEN DEBOUNCE ZORUNLU: Obsidian tek bir kaydetme işleminde 3-5 dosya
sistemi event'i fırlatır (yaz, rename, chmod). Debounce olmadan her
kaydetme 3-5 kez yeniden indeksleme tetikler — madde 35.1 ihlali.

Arayüz haberleşmesi: index güncellenince `vault:changed` event'i yayınlanır.
Arayüz bunu TanStack Query invalidation'ına çevirir (madde 21.3: kullanıcı
uygulamaya dokunmadan güncel veriyi görür).
*/

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

/// Anayasa madde 20.4.
const DEBOUNCE: Duration = Duration::from_millis(200);

/// Arayüze yayınlanan event adı.
pub const CHANGED_EVENT: &str = "vault:changed";

/// Vault izlemeyi kendi iş parçacığında başlatır.
///
/// Dönen `Handle` düşürülünce izleme durur — vault değiştiğinde eski
/// izleyicinin ölmesi için gereklidir.
pub struct WatcherHandle {
    _watcher: notify::RecommendedWatcher,
    _stop: mpsc::Sender<()>,
}

pub fn start(app: AppHandle, vault_root: PathBuf) -> Option<WatcherHandle> {
    let (event_tx, event_rx) = mpsc::channel::<notify::Result<Event>>();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    let mut watcher = notify::recommended_watcher(move |res| {
        // Kanal kapandıysa gönderim başarısız olur; sessizce yut.
        let _ = event_tx.send(res);
    })
    .ok()?;

    watcher.watch(&vault_root, RecursiveMode::Recursive).ok()?;

    std::thread::spawn(move || {
        debounce_loop(app, vault_root, event_rx, stop_rx);
    });

    Some(WatcherHandle {
        _watcher: watcher,
        _stop: stop_tx,
    })
}

/// Event'leri 200 ms penceresinde toplar, sonra tek seferde işler.
fn debounce_loop(
    app: AppHandle,
    vault_root: PathBuf,
    event_rx: mpsc::Receiver<notify::Result<Event>>,
    stop_rx: mpsc::Receiver<()>,
) {
    loop {
        if stop_rx.try_recv() != Err(mpsc::TryRecvError::Empty) {
            return; // handle düşürüldü
        }

        // İlk event'i bekle (uzun timeout: boşta CPU harcamayız — madde 5).
        let Ok(first) = event_rx.recv_timeout(Duration::from_millis(500)) else {
            continue;
        };

        let mut paths: Vec<PathBuf> = Vec::new();
        collect(&mut paths, first);

        // Debounce penceresi: pencere içinde gelen her event eklenir.
        let deadline = std::time::Instant::now() + DEBOUNCE;
        while let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
            match event_rx.recv_timeout(remaining) {
                Ok(event) => collect(&mut paths, event),
                Err(_) => break,
            }
        }

        if paths.is_empty() {
            continue;
        }

        paths.sort();
        paths.dedup();

        if apply(&app, &vault_root, &paths) {
            // Madde 21.3: arayüz kullanıcı dokunmadan tazelenir.
            let _ = app.emit(CHANGED_EVENT, ());
        }
    }
}

fn collect(paths: &mut Vec<PathBuf>, event: notify::Result<Event>) {
    let Ok(event) = event else { return };

    // Metadata-only event'ler (access time) ilgilendirmez.
    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
        return;
    }

    for path in event.paths {
        if is_relevant(&path) {
            paths.push(path);
        }
    }
}

/// Yalnız markdown, yalnız artefakt olmayan (madde 20.5).
///
/// Silinen dosya için `is_file()` false döner; bu yüzden uzantıya bakılır,
/// var olup olmadığına bakılmaz — silme de işlenmesi gereken bir değişimdir.
fn is_relevant(path: &Path) -> bool {
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) {
        return false;
    }
    // Yol üzerindeki HERHANGİ bir bileşen artefakt/gizliyse atla
    // (.obsidian/, .trash/, .git/ içindeki her şey).
    path.components().any(|c| {
        let name = c.as_os_str().to_string_lossy();
        crate::vault::reader::is_sync_artifact(&name)
    }) == false
}

/// Değişen yolları index'e uygular. Gerçekten bir şey değiştiyse `true`.
fn apply(app: &AppHandle, vault_root: &Path, paths: &[PathBuf]) -> bool {
    use tauri::Manager;

    let Some(state) = app.try_state::<crate::AppState>() else {
        return false;
    };

    let mut changed = false;
    for path in paths {
        // Madde 15.6: sync_one hash'i karşılaştırır, aynıysa hiçbir şey yapmaz.
        if state.index.sync_one(vault_root, path).unwrap_or(false) {
            changed = true;
        }
    }
    changed
}
