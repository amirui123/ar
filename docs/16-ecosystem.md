# 🌍 ۱۶ — اکوسیستم، ابزارها و کتابخانه‌ها

> این پروژه **صفر وابستگی** است تا هر خطش را بفهمی. ولی در دنیای واقعی، در
> ۹۹٪ پروژه‌ها از این جعبه‌ابزار استفاده می‌کنی. اینجا با نقشهٔ راه انتخاب.

---

## ۱) ابزارهای پایه (همیشه)

```bash
cargo       # build / test / run / doc / bench / publish / metadata / tree / add
rustup      # toolchain / target / component / update
rustfmt     # فرمتکننده
clippy      # لینتر (۲۵۰+ لینت)
rust-analyzer  # LSP برای IDE
cargo doc --open
miri        # (nightly) شکار UB
```

### نصب سریع بسته‌ها (سودمندترین‌ها)
```bash
cargo install cargo-edit        # cargo add / rm / upgrade
cargo install cargo-expand      # دیدن کد پس از ماکروها
cargo install cargo-watch       # cargo watch -x test
cargo install cargo-audit       # بررسی آسیب‌پذیری وابستگی‌ها
cargo install cargo-deny        # مجوز/نسخه/منبع وابستگی‌ها
cargo install cargo-outdated    # وابستگی‌های قدیمی
cargo install cargo-bloat       # چه چیزی باینری را بزرگ می‌کند
cargo install cargo-udeps       # وابستگی بی‌استفاده (nightly)
cargo install cargo-llvm-lines  # زمان کامپایل به تفکیک نوع
cargo install cargo-nextest     # تست‌ران سریع‌تر و خواناتر
cargo install typos-cli         # غلط‌یاب املا در کد/کامنت
cargo install just              # اجراکنندهٔ دستورات (جای make)
cargo install sccache           # کش کامپایل
cargo install hyperfine         # بنچمارک CLI
```

---

## ۲) دسته‌بندی کتابخانه‌ها (با انتخاب پیشنهادی)

### سریال‌سازی و داده
| crate | کاربرد | نکته |
|-------|--------|------|
| `serde` + `serde_json` | استاندارد طلایی JSON | `#[derive(Serialize, Deserialize)]` |
| `toml` | تنظیمات | همان الگوی serde |
| `bincode` / `postcard` | باینری سریع | برای پروتکل/ذخیره‌سازی |
| `ron` | Rust Object Notation | خوانا |
| `csv` | CSV | با serde |
| `quick-xml` / `yaml-rust2` | XML/YAML | |
| `clap` | آرگومان‌های CLI (derive) | جایگزین دستی `cli.rs` ما |
| `config` / `figment` | تنظیمات چندمنبعی | |

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Task { id: u64, title: String, done: bool }
let s = serde_json::to_string_pretty(&task)?;
let t: Task = serde_json::from_str(&s)?;
```
🎯 Lesson-learned پروژهٔ ما: **hand-rolled** `json.rs` فقط برای عدم‌وابستگی است؛
در پروژهٔ واقعی `serde_json` بزن (سریع‌تر و کامل‌تر).

### خطا
| crate | کاربرد |
|-------|--------|
| `thiserror` | خطای کتابخانه‌ای با `#[derive(Error)]` |
| `anyhow` | خطا در اپلیکیشن با `.context("…")` |
| `miette` / `color-eyre` | خطای زیبا با نمودار |
| `snafu` | خطای زمینه‌دار |

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("تسک {0} وجود ندارد")]
    NotFound(u64),
    #[error("خطای فایل: {0}")]
    Io(#[from] std::io::Error),      // ← ? خودکار
}
```

### async و شبکه
| crate | کاربرد |
|-------|--------|
| `tokio` | runtime async استاندارد |
| `async-std` / `smol` | جایگزین‌ها |
| `hyper` / `axum` / `actix-web` / `rocket` | HTTP (axum محبوب‌ترین) |
| `reqwest` | HTTP client |
| `tonic` | gRPC |
| `tower` | میان‌افزار (retry, timeout, rate-limit) |
| `tracing` + `tracing-subscriber` | لاگ ساختاریافتهٔ async (بهتر از `log`) |
| `futures` | ابزارهای Future/Stream |
| `bytes` | بافر بدون کپی |

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .route("/tasks", axum::routing::get(list_tasks));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```
🎯 سرور `web.rs` ما (std-only) را می‌توانی نقطه‌به‌نقطه با `axum` جایگزین کنی و
فرق را حس کنی: ۱۰۰ خط دستی → ۱۵ خط.

### موازی‌سازی و هم‌روندی
| crate | کاربرد |
|-------|--------|
| `rayon` | موازی‌سازی داده‌ای (سه خط) |
| `crossbeam` | کانال/ساختارهای بی‌قفل بهتر از std |
| `parking_lot` | `Mutex`/`RwLock` سریع‌تر و بدون poisoning |
| `dashmap` | `HashMap` هم‌روند |
| `once_cell` | (قدیمی‌تر از `OnceLock`/`LazyLock` std) |
| `loom` | تست مدل‌های هم‌روندی |

### کلکسیون و الگوریتم
`itertools` (زنجیره‌های راحت‌تر) · `indexmap` (ترتیب درج) · `smallvec` ·
`arrayvec` · `bitflags` · `regex` · `glob` · `walkdir` · `ignore` (مثل git) ·
`nom`/`winnow`/`pest`/`chumsky` (پارسر) · `unicode-segmentation` ·
`unicode-width` (عرض نمایش فارسی سرخود!) · `nalgebra`/`ndarray` (ریاضی) ·
`rand` (تصادفی) · `uuid` · `chrono`/`time`/`jiff` (تاریخ) ·
`base64` · `hex` · `sha2`/`blake3`/`ring`/`rustls` (رمز) ·
`flate2`/`zstd`/`tar`/`zip` · `image`/`png`/`resvg` · `polars`/`arrow` (داده) ·
`rusqlite`/`sqlx`/`diesel` (دیتابیس) · `notify` (پایش فایل) ·
`indicatif` (progress bar فارسیپسند) · `console`/`crossterm`/`ratatui` (TUI) ·
`dialoguer`/`inquire` (پرسش تعاملی).

### تست و کیفیت
| crate | کاربرد |
|-------|--------|
| `proptest` / `quickcheck` | تست ویژگی‌محور (تولید ورودی تصادفی) |
| `rstest` | fixture و پارامتری‌سازی |
| `insta` | تست اسنپ‌شات |
| `mockall` | mock کردن traitها |
| `criterion` | بنچمارک |
| `fake` / `arbitrary` | داده‌ی ساختگی/fuzz |
| `cargo-fuzz` | fuzzing |
| `assert_cmd` / `predicates` | تست CLI (خروجی، کد بازگشت) |
| `tempfile` | فایل/پوشهٔ موقت در تست |
| `serial_test` | تست‌های غیرموازی |

### سیستم و FFI
`libc` · `nix` (سیستم‌عامل یونیکس) · `windows` crate · `bindgen`/`cbindgen` ·
`pyo3` (Rust ↔ Python) · `neon`/`napi-rs` (Rust ↔ Node) · `jni` (Java) ·
`wasm-bindgen` (WebAssembly) · `cc` (کامپایل C در build.rs) ·
`memmap2` · `sysinfo` · `notify`.

### CLI
`clap` (استاندارد) · `argh` (سبک) · `pico-args` (کمینه) ·
`anstyle`/`owo-colors`/`colored`/`yansi` (رنگ) ·
`comfy-table`/`tabled`/`prettytable` (جدول) ·
`unicode-width` (عرض واقعی متن برای جدول فارسی — ما دستی حلش کردیم) ·
`indicatif` (نوار پیشرفت) · `console` (ترمینال) · `atty`/`is-terminal` ·
`humantime` (زمان خوانا) · `bytesize`.

---

## ۳) نقشهٔ انتخاب سریع

| می‌خواهی… | انتخاب |
|-----------|--------|
| CLI با زیرفرمان و help خوب | `clap` با derive |
| سرور HTTP سریع | `axum` + `tokio` + `tower` |
| تماس با API | `reqwest` + `serde` + `tokio` |
| الگوی تکراری در استخر ترد خودت | `rayon` |
| دیتابیس | `sqlx` (async) / `rusqlite` (همگام) |
| لاگ حرفه‌ای | `tracing` + `tracing-subscriber` |
| خطای کتابخانه | `thiserror` (+ `miette` برای نمایش) |
| خطای اپ | `anyhow` |
| JSON سریع با فرمت پیچیده | `serde_json` (یا `simd-json`) |
| تنظیمات | `figment`/`config` + serde |
| تست پارامتری | `rstest` + `proptest` |
| GUI | `egui`/`iced`/`slint`/`tauri` (وب‌فرانت) |
| بازی | `bevy` |
| علم داده | `polars` + `ndarray` |
| WebAssembly | `wasm-bindgen` + `wasm-pack` |

---

## ۴) مستندات و یادگیری — جعبه‌ابزار واقعی

### کتاب‌های آنلاین (رایگان)
* **The Rust Programming Language** («کتاب») — doc.rust-lang.org/book
* **Rust by Example** — doc.rust-lang.org/rust-by-example
* **The Rustonomicon** — unsafe و مباحث داخلی
* **Rust for Rustaceans** (Jon Gjengset) — سطح حرفه‌ای (خریداری)
* **Rust Atomics and Locks** (Mara Bos) — رایگان آنلاین: marabos.nl/atomics
* **The Async Book** — rust-lang.github.io/async-book
* **The Performance Book** — nnethercote.github.io/perf-book
* **Cookbook** — rust-lang-nursery.github.io/rust-cookbook
* **Writing an OS in Rust** (Philipp Oppermann) — سطح پایین
* **Comprehensive Rust** (Google) — دورهٔ گوگل
* **Rust Design Patterns** — rust-unofficial.github.io/patterns
* **Idiomatic Rust** (مثال‌های اصیل) — github.com/mre/idiomatic-rust

### مرجع
* `docs.rs/<crate>` — مستندات هر crate
* `std` — doc.rust-lang.org/std (هر صفحه مثال + تست دارد)
* **cheats.rs** — چیت‌شیت Rust
* **blessed.rs** — «برای این کار چه crateی؟» — عالی برای انتخاب
* **lib.rs** — مرورگر crateها (بهتر از crates.io برای کشف)
* **Rust API Guidelines** — checklist طراحی API
* **RustSec Advisory DB** — آسیب‌پذیری‌ها
* **This Week in Rust** — خبرنامهٔ هفتگی

### تمرین
* **rustlings** — ۹۰+ تمرین کوتاه (بهترین شروع)
* **Advent of Code** با Rust
* **Exercism Rust track**
* **Hecto** (ساخت ادیتور متن) · **Command-Line Rust** (ساخت ابزارهای C)
* **Coding Challenges** (John Crickett) — ساختن ابزارهای واقعی

### ابزار تعاملی
* **Rust Playground** — play.rust-lang.org
* **Godbolt** — دیدن اسمبلی خروجی
* **QuickType** — تولید نوع از JSON
* **cargo-wizard** — تولید پروفایل بهینه

---

## ۵) مسیر پیشرفت (بعد از این پروژه)

| سطح | چه بلد باشی |
|-----|-------------|
| ۱. پایه | مالکیت/قرض، `struct`/`enum`, `match`, `Vec`/`String`, `Result`/`Option` |
| ۲. عملی | ماژول، Cargo، تست، خطای سفارشی، trait |
| ۳. اصیل | ایتراتور، `impl Trait`، بستار، `Cow`, نیو‌تایپ |
| ۴. حرفه‌ای | `unsafe`, ماکرو، `Send`/`Sync`, async، بنچمارک |
| ۵. معمار | طراحی API، معیارهای سازگاری، انتشار crate، طراحی بدون‌وابستگی |

🎯 **پیشنهاد پروژه‌های بعدی برای تثبیت:**
1. یک `grep` ساده با `walkdir` + `regex` + `rayon` (مقایسهٔ سرعت با `rg`).
2. یک سرور HTTP با `axum` + `sqlx` (همان `ar` ولی دیتابیس).
3. یک مفسر/ماشین حساب با `chumsky` یا دست‌ساز (پارسر).
4. یک TUI تسک‌منیجر با `ratatui`.
5. انتشار اولین crate روی crates.io.

[⬅️ قبلی: چیت‌شیت std](15-std-cheatsheet.md) · [بعدی: واژه‌نامه ➡️](17-glossary.md)
