/*!
Index katmanı — Anayasa madde 9, 15.

Bu modül TÜRETİLMİŞ VERİ tutar. Kaynak Obsidian'dır (madde 7.2).
Vault → index akışı TEK YÖNLÜDÜR (madde 15.4): index'ten vault'a asla
veri akmaz.

Sprint 0: şema + bağlantı hazır, tablolar boş.
Sprint 1: `builder` modülü eklenecek — tam tarama ve artımlı güncelleme.
*/

mod db;
mod schema;

pub use db::IndexHandle;
pub use schema::SCHEMA_VERSION;
