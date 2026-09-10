/*!
Hermes katmanı — Anayasa madde 18, 19.

Hermes uygulamanın BEYNİDİR (madde 7.1) ama ARKELÉS için sadece bir
uzak servistir. Bu modül o servisin tek kapısıdır.

Anayasa madde 19.3 gereği bu modülün ürettiği hiçbir sır (token, adres)
webview'e sızmaz — dışarıya yalnız `HermesHealth` gibi zararsız özetler çıkar.
*/

mod client;

pub use client::{health, HermesHealth};
