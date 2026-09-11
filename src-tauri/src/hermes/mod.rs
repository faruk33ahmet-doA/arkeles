/*!
Hermes katmanı — Anayasa madde 18, 19; Sprint 4, Sprint 5.

Hermes uygulamanın BEYNİDİR (madde 7.1) ama ARKELÉS için bir uzak servistir.
Bu modül o servisin tek kapısıdır.

  contract     → aksiyon allowlist'i, token, istem metni
  capabilities → Hermes-native ilanları ARKELÉS semantik yeteneklerine eşler
  http         → REST (health, status, toolsets, skills)
  rpc      → JSON-RPC over WebSocket kanalı + hata sınıflandırması
  turn     → gerçek Hermes tur protokolü (Hermes'e özgü adların tek yeri)
  jobs     → ARKELÉS'in gönderdiği işlerin defteri
  client   → yüksek seviye: sağlık + yetenek

Madde 19.3 gereği bu modülün ürettiği hiçbir sır (token, adres) webview'e
sızmaz — dışarıya yalnız zararsız özetler çıkar.
*/

pub mod capabilities;
pub mod contract;
mod http;
pub mod jobs;
mod rpc;
pub mod turn;

mod client;

#[cfg(test)]
mod fake;

pub use client::{health, summary};
