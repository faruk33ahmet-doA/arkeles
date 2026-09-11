/*!
Hermes katmanı — Anayasa madde 18, 19; Sprint 4.

Hermes uygulamanın BEYNİDİR (madde 7.1) ama ARKELÉS için bir uzak servistir.
Bu modül o servisin tek kapısıdır.

  contract → gerçek API sözleşmesi ve yetenek allowlist'i
  http     → REST (health, status, sessions)
  rpc      → JSON-RPC over WebSocket (iş gönderme)
  jobs     → ARKELÉS'in gönderdiği işlerin defteri
  client   → yüksek seviye: sağlık + yetenek

Madde 19.3 gereği bu modülün ürettiği hiçbir sır (token, adres) webview'e
sızmaz — dışarıya yalnız zararsız özetler çıkar.
*/

pub mod contract;
mod http;
pub mod jobs;
mod rpc;

mod client;

pub use client::{health, summary};
