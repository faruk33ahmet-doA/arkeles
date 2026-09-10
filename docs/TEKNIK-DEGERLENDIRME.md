# ARKELÉS V1 — Teknik Değerlendirme Raporu

**Tarih:** 2026-09-11
**Durum:** Kod yazılmadı. Bu rapor geliştirme öncesi teknik karar dokümanıdır.
**Kapsam:** Mimari değerlendirme, risk analizi, iyileştirme önerileri, anayasa değişiklik talepleri, Sprint 1 planı.

---

## 0. Özet Yargı

Anayasa **sağlam bir ürün felsefesi** tanımlıyor ve en önemli kararı doğru veriyor:
*sorumluluk ayrımı*. Hermes üretir, Obsidian saklar, ARKELÉS gösterir. Bu ayrım
projenin en değerli mimari kararı ve korunmalı.

Ancak anayasa **veri katmanı hakkında hiçbir şey söylemiyor** ve iki yerde
kendi kendisiyle çelişiyor. Kod yazmadan önce çözülmesi gereken 4 kritik karar var:

| # | Karar | Neden şimdi? |
|---|---|---|
| K1 | ARKELÉS yazma yetkisi var mı, yoksa salt-okunur mu? | Tüm veri katmanını belirler |
| K2 | Markdown doğrudan mı okunacak, index üzerinden mi? | Performans anayasasını belirler |
| K3 | Not kimliği (ID) nasıl tanımlanacak? | Sonradan değiştirmesi en pahalı karar |
| K4 | Hermes API sözleşmesi ve güven sınırı | Güvenlik + kırılganlık |

Bu 4 karar verilmeden yazılan kod, 3. sprintte tamamen yeniden yazılır.

---

## 1. Mimari Değerlendirme

### 1.1 Doğru olan kararlar (dokunulmayacak)

**Sorumluluk ayrımı.** "Hermes çalışır, Obsidian saklar, ARKELÉS gösterir" cümlesi,
çoğu kişisel-OS projesinin battığı yerde doğru tarafta duruyor. Tek uygulama
hem AI çalıştırıp hem UI render etmeye kalkarsa ne hızlı ne bakımı yapılabilir olur.
Bu ayrım korunacak.

**Obsidian'ın tek doğruluk kaynağı olması.** Veri, uygulamadan bağımsız
düz metin dosyalarında yaşıyor. Bu:
- Uygulama çökse/silinse veri kaybı yok
- Başka araçlarla (grep, git, Obsidian eklentileri) erişilebilir
- Vendor lock-in yok
- Yedekleme bedava

Bu, uzun ömürlü kişisel sistemler için doğru karardır.

**Desktop-first + tamamen lokal.** Sağlık ve Finans modülleri var. Bu veri
buluta çıkmamalı. Lokal tercih burada güvenlik kararıdır, sadece performans kararı değil.

**Tauri seçimi.** Electron yerine Tauri: ~10 MB bundle, ~80–150 MB RAM
(Electron'da ~400 MB+), native webview. "RAM minimum" kuralıyla uyumlu tek doğru seçim.
Ayrıca Rust tarafı, dosya sistemi ve index işi için gerçek bir avantaj — bu iş
JavaScript'te yapılırsa yavaş olur.

**Alt katmanlarda zoom kullanılmaması.** Anayasada "sadece ana katmanlarda zoom"
denmiş. Bu isabetli bir kısıtlama; sınırsız zoom hiyerarşisi kullanıcıda
yön kaybı yaratır ve performansı öldürür.

### 1.2 Eksik olan katmanlar

Anayasa 4 katmandan 2'sini tanımlıyor:

```
┌─────────────────────────────────────────┐
│  Sunum katmanı        React + Tailwind  │  ✅ tanımlı
├─────────────────────────────────────────┤
│  Durum katmanı        ???               │  ❌ tanımsız
├─────────────────────────────────────────┤
│  Veri erişim katmanı  ???               │  ❌ tanımsız
├─────────────────────────────────────────┤
│  Kaynak               Obsidian + Hermes │  ✅ tanımlı
└─────────────────────────────────────────┘
```

Ortadaki iki katman tanımlanmadan yazılan kod, "her component kendi dosyasını
okur" noktasına düşer. Bu, performans anayasasının ihlalidir.

### 1.3 Önerilen tam mimari

```
                    ARKELÉS (Tauri uygulaması)
┌───────────────────────────────────────────────────────────┐
│  WEBVIEW (React)                                          │
│                                                           │
│   Sunum:   Katmanlar · Dock · Zoom Trail · Paneller       │
│   Durum:   Zustand (UI durumu)  ·  TanStack Query (veri)  │
│            └─ tek IPC yüzeyi, component'ler dosya bilmez  │
└──────────────────────────┬────────────────────────────────┘
                           │  Tauri IPC (tip güvenli komutlar)
┌──────────────────────────┴────────────────────────────────┐
│  ÇEKİRDEK (Rust)                                          │
│                                                           │
│   ┌─────────────┐   ┌──────────────┐   ┌───────────────┐   │
│   │ Vault Okur  │──▶│ SQLite Index │◀──│ Dosya İzleyici│   │
│   │ (md parser) │   │ (+ FTS5)     │   │ (notify)      │   │
│   └─────────────┘   └──────┬───────┘   └───────────────┘   │
│                            │ türetilmiş, her an silinebilir │
│   ┌─────────────┐          │                                │
│   │ Vault Yazar │  mekanik mutasyonlar (atomic write)       │
│   └─────────────┘                                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ Hermes İstemcisi — token burada, webview'de DEĞİL   │   │
│   └─────────────────────────────────────────────────────┘   │
└──────────┬────────────────────────────────┬───────────────┘
           │ dosya sistemi                  │ 127.0.0.1 + token
    ┌──────┴───────┐                 ┌──────┴──────┐
    │ Obsidian     │◀────yazar───────│  HERMES     │
    │ Vault (.md)  │                 │  (daemon)   │
    │ = TEK GERÇEK │                 └─────────────┘
    └──────────────┘
```

**Kritik nüans:** SQLite bir *veri kaynağı değil*, türetilmiş bir **projeksiyon**.
Silinirse vault'tan 1 saniyede yeniden kurulur. "Obsidian tek doğruluk kaynağıdır"
kuralı ihlal edilmiyor — index, gerçeğin hızlı bir kopyasıdır, gerçeğin kendisi değil.

---

## 2. Risk Analizi

Riskler etki × olasılık sırasına göre.

### R1 — Markdown'ı doğrudan okumak · **KRİTİK**

`Bugün` ekranı "bugün ne var" sorusunu cevaplamak için vault'un tamamını taramak
zorunda. 2.000 notluk bir vault'ta:

| Yaklaşım | Süre (soğuk) | Süre (sıcak) |
|---|---|---|
| JS'te tüm dosyaları oku + parse et | 3–8 sn | 1–3 sn |
| Rust'ta tüm dosyaları oku + parse et | 400–900 ms | 200 ms |
| Rust + SQLite index sorgusu | **1–5 ms** | **1–5 ms** |

"Her etkileşim anlık hissedilmelidir" kuralı, index olmadan matematiksel olarak
sağlanamaz. Bu bir optimizasyon değil, **mimari zorunluluk**.

> **Çözüm:** Rust çekirdeğinde `rusqlite` + FTS5 index. Vault → index tek yönlü.
> Index dosyası app-data dizininde (vault'un İÇİNDE değil — vault kirlenmesin).
> Şema versiyonu değişince index otomatik yeniden kurulur.

### R2 — Üç yazar aynı dosyada · **KRİTİK**

Obsidian, Hermes ve ARKELÉS aynı dosyalara dokunuyor. Somut arıza senaryoları:

1. Obsidian bir notu kaydederken ARKELÉS yarısını okur → parse hatası veya bozuk veri
2. ARKELÉS bir checkbox'ı işaretlerken Hermes aynı dosyayı yeniden yazar → kayıp yazma
3. iCloud/Dropbox senkronu `.md.icloud` placeholder veya `dosya 2.md` çakışma kopyası üretir
4. Dosya izleyici, Obsidian'ın tek kaydında 3–5 event fırlatır → gereksiz reindex fırtınası

> **Çözüm paketi:**
> - **Atomic write:** geçici dosyaya yaz → `fsync` → `rename`. Yarım dosya asla görünmez.
> - **Optimistic concurrency:** ARKELÉS yazarken, okuduğu andaki `mtime + içerik hash`'ini
>   doğrular. Uyuşmazsa yazmaz, kullanıcıya çakışma gösterir. **Sessizce üzerine yazmak yasak.**
> - **Debounce:** izleyici event'leri 200 ms pencerede birleştirilir.
> - **Hash karşılaştırma:** içerik hash'i aynıysa reindex yapılmaz (kendi yazımızı yeniden okumayız).
> - **Sync artifact filtresi:** `.icloud`, `conflicted copy`, `~$` desenleri yok sayılır.
> - **Vault git repo olmalı.** Bedava, bulut gerektirmez, her arızada geri dönüş noktası verir.

### R3 — Hermes Local API güven sınırı · **YÜKSEK**

Kimlik doğrulaması olmayan bir `localhost` HTTP sunucusu, makinedeki **her süreç ve
her web sayfası** tarafından çağrılabilir. Tarayıcıda açık bir sekme
`fetch('http://127.0.0.1:PORT/...')` ile Hermes'e komut verebilir. Hermes; Telegram'a
mesaj atan, belge üreten, Obsidian'a yazan bir bileşen. Bu kabul edilebilir bir yüzey değil.

> **Çözüm:**
> - Hermes yalnızca `127.0.0.1`'e bind edilir (`0.0.0.0` asla).
> - Paylaşılan token, `~/.arkeles/hermes.token` içinde `0600` izinle tutulur.
> - Hermes `Origin` header'ı olan istekleri reddeder (tarayıcı kaynaklı çağrı engellenir).
> - **Hermes'e webview'den asla doğrudan istek atılmaz.** Tüm çağrılar Rust çekirdeği
>   üzerinden proxy'lenir. Token JavaScript'e hiç inmez.
> - Tauri CSP'de `connect-src` yalnızca IPC'ye açık kalır.

### R4 — Zoom Trail performansı ve "bekletmeme" çelişkisi · **YÜKSEK**

Anayasa iki şey istiyor: (a) sayfa değişimi hissi olmasın, (b) animasyon kullanıcıyı
bekletmesin. Naif bir zoom implementasyonu ikisini birden ihlal eder: iki katman
aynı anda mount olur, React ağacı ikiye katlanır, 120 Hz ekranda 8,3 ms bütçe aşılır, jank olur.

> **Çözüm — Hareket Anayasası (ihlali kod review'da red sebebi):**
> - Yalnızca `transform` ve `opacity` animate edilir. `width/height/top/left/filter/blur`
>   büyük yüzeylerde **yasak** (layout ve paint tetikler).
> - Geçiş süresi 180–260 ms. Üst sınır 300 ms.
> - **Hedef katman, animasyonun 1. karesinde etkileşime hazırdır.** Animasyon dekorasyondur,
>   kapı değildir. Kullanıcı animasyon ortasında tıklarsa işlem geçer.
> - Veri, animasyon **başlamadan önce** prefetch edilir. Zoom sırasında spinner görülmez.
> - Kaynak katman, geçiş bitince unmount edilir. İki katman kalıcı olarak mount kalmaz.
> - `prefers-reduced-motion` desteklenir → süre 0.
> - En fazla 2 zoom seviyesi. Daha derini panel geçişidir (anayasayla uyumlu).

### R5 — MVP kapsamı gerçekçi değil · **YÜKSEK**

Anayasa 9 ana modül + İş altında 7 çalışma alanı = **16 yüzey** tanımlıyor.
Bu bir MVP değil, 6 aylık bir yol haritası. "Önce sağlam temel oluştur" kuralıyla çelişiyor.

> **Çözüm:** Dock **9 modülün tamamını gösterir** (zihinsel model bütün kalsın),
> ama Sprint 1–3'te yalnızca **3'ü gerçek**: `Bugün`, `İş` (yalnız 2 çalışma alanı), `Sistem`.
> Diğerleri görünür ama "yakında" durumunda — tıklanabilir, boş durum ekranı gösterir.
> Yarım yapılmış 16 modül, tam yapılmış 3 modülden kötüdür.

### R6 — shadcn/ui varsayılan görünümü · **ORTA**

shadcn/ui iyi bir davranış kütüphanesi ama varsayılan teması, AI ile üretilmiş
her dashboard'un görünümü. "Apple seviyesinde premium, sessiz, profesyonel" hedefiyle
doğrudan çelişir. Bu bir teknik risk değil, **ürün kimliği riski** — ve geri dönüşü pahalı.

> **Çözüm:** shadcn yalnızca **davranış primitifleri** için kullanılır: `Dialog`, `Popover`,
> `Command`, `Tooltip`, `ScrollArea`. Bunların focus trap / klavye / erişilebilirlik işini
> sıfırdan yazmak haftalar alır ve hatalı olur. **Görsel katman tamamen kendimizin.**
> shadcn'in varsayılan renk/gölge/radius teması kullanılmaz, `tokens.css` üzerine bindirilir.

### R7 — Not kimliği (ID) stratejisi · **ORTA (ama geri dönüşü en pahalı)**

Notlara dosya adıyla referans verilirse, ilk yeniden adlandırmada tüm ilişkiler kopar.
Bu hata 6. ayda fark edilirse migration acı verir.

> **Çözüm:** Her yönetilen not, frontmatter'da **kalıcı bir `arkeles_id`** taşır
> (ULID — sıralanabilir, çakışmaz). Dosya adı sadece insan içindir, kimlik değildir.
> Index bu ID üzerinden çalışır. Dosya taşınsa/adı değişse ilişkiler ayakta kalır.

### R8 — Hermes sözleşme kırılganlığı · **ORTA**

ARKELÉS ve Hermes bağımsız gelişecek. Sözleşme yazılı değilse ARKELÉS sessizce bozulur.

> **Çözüm:** `GET /health` endpoint'i sürüm + yetenek listesi döner:
> `{ "version": "1.2.0", "capabilities": ["pdf", "telegram", "ai.summarize"] }`
> ARKELÉS, desteklenmeyen yetenekleri **UI'da hiç göstermez**. Hermes kapalıysa uygulama
> tam çalışır, yalnızca "üretim" aksiyonları pasifleşir ve dock'ta durum göstergesi gri olur.

### R9 — Gereksiz render · **ORTA**

React'te en yaygın performans ihlali: tek Context'te tutulan uygulama durumu.
Tek alan değişince ağacın tamamı render olur.

> **Çözüm:** Sıcak veri Context'te tutulmaz. Zustand'ın selector'lı aboneliği kullanılır
> (yalnızca ilgili component render olur). Liste öğeleri `React.memo` + kararlı key.
> 200+ satırlı listelerde sanallaştırma (`@tanstack/react-virtual`).
> React DevTools "highlight updates" ile sprint sonunda görsel doğrulama.

### R10 — Rust toolchain kurulu değil · **DÜŞÜK ama bloke edici**

Bu makinede `cargo` ve `rustc` yok. Tauri buildlenemez. Sprint 0'ın ilk maddesi.

### R11 — Framer Motion runtime maliyeti · **DÜŞÜK**

Tam paket ~50 KB gzip ve gereksiz feature'lar taşıyor.

> **Çözüm:** `LazyMotion` + `domAnimation` feature bundle ile içe alınır.
> Runtime ~%60 küçülür, `motion` ve `AnimatePresence` API'si aynı kalır.

### R12 — Hassas veri diskte açık · **DÜŞÜK (ama not edilmeli)**

Sağlık ve Finans verisi düz metin. Lokal olması iyi ama:
> FileVault açık olmalı. Vault içeriği hiçbir log satırına yazılmaz
> (crash raporu, konsol, dosya). Log'da yalnızca ID ve dosya yolu bulunur.

---

## 3. Anayasa Değişiklik Önerileri

Aşağıdaki 7 madde, anayasanın ruhunu bozmadan çelişkilerini gideriyor.
Her biri gerekçelidir ve onayınıza sunulur.

### A1 — "Server yok" ifadesi düzeltilmeli

**Mevcut:** "Server yok"
**Öneri:** "**Uzak** server yok. Hermes lokal bir daemon'dur ve mimarinin parçasıdır."
**Gerekçe:** Hermes bir Local API sunuyor — teknik olarak bir sunucudur. Mevcut ifade
anayasayı kendi kendisiyle çelişkiye düşürüyor. Kastedilen "bulut yok" ve bu doğru;
sadece doğru kelimeyle yazılmalı ki sonraki kararlar bu çelişkiye çarpmasın.

### A2 — "ARKELÉS veri üretmez" netleştirilmeli

**Mevcut:** "ARKELÉS veri üretmez"
**Öneri:** "ARKELÉS **yeni bilgi** üretmez. **Türetilmiş** veri (index, önbellek, görünüm
durumu) üretebilir; bunlar her an silinip vault'tan yeniden kurulabilir olmak zorundadır."
**Gerekçe:** Index olmadan performans hedefi tutulamaz (bkz. R1). Ama "türetilmiş"
tanımı, Obsidian'ın tek gerçek olma statüsünü koruyor. Test şu: *index dosyasını silince
hiçbir bilgi kaybolmuyor mu?* Cevap "evet" kalmak zorunda.

### A3 — Yazma yetkisi ikiye bölünmeli · **EN ÖNEMLİ MADDE**

**Mevcut:** "ARKELÉS sadece gösterir, sadece yönetir" — ama "yönetmek" yazmak demektir.
Bu tanımsız.
**Öneri — iki sınıf mutasyon:**

| Sınıf | Örnek | Yetkili |
|---|---|---|
| **Mekanik** | Checkbox işaretle, frontmatter alanı değiştir, sırayı değiştir, etiket ekle | **ARKELÉS** (Rust yazar, atomic + hash doğrulamalı) |
| **Semantik** | Belge üret, PDF çıkar, rapor yaz, AI çalıştır, Telegram gönder, yeni not oluştur | **Yalnızca Hermes** |

**Gerekçe:** Her yazmayı Hermes'e göndermek iki sorun yaratır: (1) Hermes kapalıyken
uygulama salt-okunur bir vitrine dönüşür — bir "kişisel işletim sistemi" için kabul
edilemez; (2) checkbox işaretlemek için IPC + HTTP + dosya yazma zinciri, "anlık hissettirme"
kuralını ihlal eder. Bu ayrım felsefeyi korur: ARKELÉS hâlâ *iş yapmıyor*, sadece
kullanıcının doğrudan niyetini dosyaya yansıtıyor. Üretim tekeli Hermes'te kalıyor.

### A4 — Durum ve veri katmanı teknolojiye eklenmeli

**Öneri:** Varsayılan teknoloji listesine eklenir:
- **Zustand** — UI durumu (aktif katman, dock, panel durumları). ~1 KB, selector'lı abonelik.
- **TanStack Query** — veri katmanı (cache, invalidation, stale-while-revalidate).

**Gerekçe:** "Gereksiz render yasaktır" kuralı ancak seçmeli abonelikle uygulanabilir.
TanStack Query'nin stale-while-revalidate davranışı, "anlık hissetme" hedefinin
doğrudan aracıdır: eski veri hemen gösterilir, arkada tazelenir, kullanıcı hiç beklemez.
Redux gereksiz ağır; Context sıcak veri için yanlış araç.

### A5 — shadcn/ui kapsamı sınırlandırılmalı

**Öneri:** "shadcn/ui **yalnızca davranış primitifleri** için kullanılır.
Görsel katman ARKELÉS'e özgüdür. shadcn varsayılan teması kullanılmaz."
**Gerekçe:** bkz. R6. Premium hedefi ile hazır tema doğrudan çelişir.

### A6 — MVP kapsamı 3 modüle indirilmeli

**Öneri:** Dock 9 modülü gösterir, MVP'de `Bugün` + `İş` (2 çalışma alanı) + `Sistem` gerçektir.
**Gerekçe:** bkz. R5. "Önce sağlam temel" kuralının doğal sonucu.

### A7 — Komut paleti (Cmd+K) MVP'ye alınmalı

**Öneri:** Cmd+K komut paleti Sprint 1 kapsamındadır, sonraya bırakılamaz.
**Gerekçe:** Bir *işletim sistemi* hissini veren şey klavyedir, fare değil. Dock keşif
içindir; günlük hız klavyeden gelir. Sonradan eklenen komut paletleri hep eksik kalır
çünkü aksiyonlar merkezi bir kayıt (action registry) üzerinden tanımlanmamış olur.
Bu kaydı baştan kurmak bedavaya yakın, sonradan kurmak refactor demektir.

### Reddedilen değişiklik önerisi (şeffaflık için)

**Tauri → Electron geçişi düşünülmedi ve önerilmiyor.** RAM ve bundle hedefleriyle
uyumsuz. Rust öğrenme maliyeti gerçek bir bedel ama karşılığında dosya sistemi ve
index performansı kazanılıyor — ki bu projenin en sıcak yolu (hot path).

---

## 4. Sprint 0 — Temel (yarım gün)

Sprint 1'in ön koşulu. Tek çıktısı: boş ama çalışan bir pencere.

1. Rust toolchain kurulumu (`rustup`) — **şu an eksik**
2. Tauri 2.x + React + TypeScript iskeleti, `pnpm`
3. Tailwind + `tokens.css` (boş ama yerinde)
4. Vault yolu konfigürasyonu (hardcode yasak, ilk açılışta sorulur)
5. `git init` + ilk commit
6. `docs/ADR/` klasörü — K1–K4 kararları birer ADR olarak yazılır

**Bitti tanımı:** `pnpm tauri dev` açılıyor, boş pencere geliyor, HMR çalışıyor.

---

## 5. Sprint 1 — "Kabuk ve Çekirdek"

**Hedef:** Gerçek vault verisini gerçek zamanlı gösteren, gezinebilen, hızlı bir kabuk.
**Süre:** 5–7 gün
**Kapsam dışı (bilinçli):** yazma, Hermes komutları, İş modülü içeriği, 6 modül

### 5.1 Rust çekirdeği (en riskli parça — önce bu yapılır)

| Görev | Detay |
|---|---|
| `vault::reader` | Frontmatter (YAML) + gövde + görev (`- [ ]`) parser'ı |
| `vault::id` | ULID üretimi, `arkeles_id` okuma. **Bu sprintte yazma yok** — ID yoksa not "yönetilmeyen" sayılır |
| `index::schema` | SQLite şema v1: `notes`, `tasks`, `links` + FTS5 sanal tablosu |
| `index::build` | Tam tarama → index. Hedef: 2.000 not < 1 sn |
| `index::watch` | `notify` crate, 200 ms debounce, hash karşılaştırma, sync-artifact filtresi |
| `hermes::client` | Yalnızca `GET /health`. Token dosyadan okunur, webview'e inmez |
| Testler | Parser için birim testler: bozuk YAML, eksik frontmatter, boş dosya, BOM, CRLF, emoji, 5 MB not |

**Neden Rust önce?** Belirsizliğin ve riskin tamamı burada. UI'yi hayali veriyle
yazıp sonra çekirdeğe bağlamak, iki kez iş yapmak demek.

### 5.2 IPC yüzeyi (dar tutulur)

```
vault_status()                     -> { path, note_count, indexed_at }
list_today()                       -> TodayView   (bugünün görevleri + notları)
list_notes(filter, limit, cursor)  -> Page<NoteSummary>
get_note(id)                       -> Note
search(query, limit)               -> Vec<SearchHit>     (FTS5)
hermes_health()                    -> HermesHealth
```

Rust tipleri `ts-rs` ile TypeScript'e üretilir. **Elle yazılmış tip tanımı yasak** —
sözleşme tek yerde yaşar, sürüklenme (drift) olmaz.

### 5.3 Tasarım sistemi (`tokens.css`) — UI'den önce

Kod yazmadan önce sabitlenir, sonra tartışılmaz:
- Uzay ölçeği: 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64
- Radius: 6 / 10 / 16
- Tipografi rampası: 11 / 13 / 15 / 20 / 28 / 40 (SF Pro / system-ui)
- Yükseklik (elevation): 3 seviye — hiçbiri renkli gölge kullanmaz
- Hareket: `--dur-fast 120ms`, `--dur-base 200ms`, `--dur-layer 260ms`,
  `--ease-out cubic-bezier(.32,.72,0,1)`
- Renk: nötr gri ölçek + **tek** vurgu rengi. Gradyan yok, neon yok, glow yok.
- Karanlık tema varsayılan, aydınlık tema token seviyesinde destekli

### 5.4 Kabuk ve gezinme

| Görev | Detay |
|---|---|
| Katman motoru | `useLayer()` — 2 seviye zoom. Kaynak katman geçiş sonunda unmount |
| Zoom geçişi | Yalnız `transform`/`opacity`, 260 ms, `prefers-reduced-motion` destekli |
| Prefetch | Zoom animasyonu başlamadan hedef katmanın verisi TanStack Query ile ısıtılır |
| Bottom Dock | 9 modül görünür, 3'ü aktif, 6'sı "yakında" boş durumu |
| Zoom Trail | Üstte breadcrumb, tıklanabilir, geri dönüş `Esc` ile de çalışır |
| Komut paleti | Cmd+K. Merkezi `actionRegistry` + FTS5 üzerinden not arama |
| Durum göstergesi | Dock'ta Hermes ve index durumu (yeşil / gri / kırmızı) |

### 5.5 Bugün modülü (salt-okunur)

Gerçek vault verisiyle çalışan tek modül:
- Bugüne ait görevler (vadesi geçenler ayrı grupta)
- Bugün oluşturulan/değiştirilen notlar
- Boş durum ekranı gerçekten tasarlanır (sonradan eklenen boş durumlar hep çirkin olur)
- **Yazma yok.** Checkbox görünür ama pasif. Yazma Sprint 2.

### 5.6 Performans bütçesi — Bitti tanımının parçası

Sprint sonunda **ölçülür ve rapor edilir**. Aşan sprint kapanmaz.

| Metrik | Bütçe |
|---|---|
| Soğuk açılış → ilk anlamlı kare | < 800 ms |
| Katman geçişi (p95 kare süresi) | < 8,3 ms (120 Hz) |
| Etkileşim → görsel tepki | < 100 ms |
| `list_today()` (2.000 notluk vault) | < 10 ms |
| Tam index kurulumu (2.000 not) | < 1 sn |
| Boşta RAM | < 180 MB |
| JS bundle (gzip) | < 250 KB |

### 5.7 Sprint 1 "Bitti" tanımı

- [ ] Gerçek Obsidian vault'undan gerçek veri okunuyor
- [ ] Obsidian'da not değiştirilince ARKELÉS < 500 ms içinde güncelleniyor (uygulamaya dokunmadan)
- [ ] 2 zoom katmanı arasında geçiş jank'sız
- [ ] Cmd+K çalışıyor, notlarda arama yapıyor
- [ ] Hermes kapalıyken uygulama tam çalışıyor, gösterge gri
- [ ] Index dosyası silinip uygulama açılıyor → kendini yeniden kuruyor, veri kaybı yok
- [ ] 7 performans bütçesinin tamamı ölçülmüş ve tutuyor
- [ ] Parser birim testleri geçiyor
- [ ] ADR'ler yazılı, K1–K4 kararları dokümante

### 5.8 Sonraki sprintler (başlık düzeyinde)

- **Sprint 2 — Yazma:** mekanik mutasyonlar, atomic write, çakışma UI'si, optimistic update
- **Sprint 3 — İş modülü:** 2 çalışma alanı, alt panel geçişleri, ilişkili not görünümü
- **Sprint 4 — Hermes komutları:** semantik aksiyonlar, iş kuyruğu, ilerleme durumu
- **Sprint 5 — Genişleme:** kalan modüller, sağlamlaştırma

---

## 6. Onayınıza Sunulan Kararlar

Kod yazmaya başlamak için gereken cevaplar:

1. **A3 (yazma yetkisi ikiye bölünmesi)** onaylanıyor mu? — En kritik madde.
2. **A4 (SQLite index)** onaylanıyor mu? — Performans hedefi buna bağlı.
3. **A6 (MVP 3 modüle indirilmesi)** onaylanıyor mu?
4. **A7 (Cmd+K Sprint 1'de)** onaylanıyor mu?
5. Obsidian vault'unun **yolu** ve yaklaşık **not sayısı** nedir?
6. Hermes şu an **çalışır durumda mı**, API'si dokümante mi? Yoksa Sprint 1'de
   yalnızca `/health` sözleşmesini varsayıp mock'la ilerlerim.
