# ⚙️ ۰۱ — نصب Rust و کاملِ cargo

## ۱) روش‌های نصب

### الف) روش رسمی (rustup) — اگر اینترنت آزاد داری
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version && cargo --version
```

rustup «مدیر نسخه‌ها» است:
```bash
rustup update                 # به‌روزرسانی به آخرین stable
rustup toolchain list         # کدام نسخه‌ها نصب‌اند
rustup install nightly        # نصب نسخهٔ نایتلی (برای ویژگی‌های آزمایشی)
rustup default stable         # نسخهٔ پیش‌فرض
rustup component add rustfmt clippy    # ابزارهای ضروری!
rustup target add wasm32-unknown-unknown   # کامپایل به WebAssembly
```
> `rustfmt` = قالب‌بندی خودکار، `clippy` = لینتر (پیشنهادهای سبک و کارایی).
> **هر دو را از روز اول نصب کن** و در CI بگذار.

### ب) 🕵️ ترفند: نصب بدون دسترسی به سرورهای رسمی (همان روشی که این پروژه نصب شد)
در بعضی محیط‌ها (sandbox، شبکهٔ بستهٔ سازمانی، فیلتر) `static.rust-lang.org`
بلاک است. اما روی **رجیستری npm و PyPI** بسته‌هایی هستند که *خودِ تاربال‌های
رسمی* Rust را داخلشان دارند:

| بسته | چه می‌دهد |
|------|-----------|
| `@rust-sandbox/rust-<MINOR>-x86_64-linux` | rustc + cargo + rust-std (کامل) |
| `@toolchain-bin/rustc-<VERSION>-<TARGET>` | تک‌تک اجزا جدا |
| `@rustbin/rust-std-<VERSION>-<TARGET>` | فقط std (برای cross-compile) |
| `rustup` (روی PyPI) | باینری rustup (ولی toolchain نمی‌آورد) |
| `@kreijstal/rubrc` | rustc در WebAssembly 😮 |

اسکریپت آمادهٔ این پروژه:
```bash
bash scripts/setup-rust.sh          # نصب در ~/.cache/rust
export PATH="$HOME/.cache/rust/bin:$PATH"
```
اسکریپت خودش SHA256 را چک می‌کند و بعد `rustc --version` می‌گیرد.

### ج) مدیر بستهٔ سیستم (ساده ولی کند پیش می‌رود)
```bash
sudo apt install rustc cargo      # دبیان/اوبونتو — معمولاً نسخهٔ قدیمی!
```
⚠️ برای edition 2024 به **1.85+** نیاز داری. نسخهٔ apt دبیان اغلب عقب است؛
همین‌جاست که راه (الف) یا (ب) برنده می‌شود.

---

## ۲) cargo — قلب ابزارها

`cargo` هم سازنده، هم مدیر بسته، هم مدیر تست، هم انتشاردهنده است.

### دستورهای روزمره
| دستور | کار |
|-------|-----|
| `cargo new <name>` | پروژهٔ تازه (binary) |
| `cargo new --lib <name>` | پروژهٔ کتابخانه |
| `cargo init` | در پوشهٔ موجود پروژه بساز |
| `cargo build` | ساخت (debug) |
| `cargo build --release` | ساخت بهینه |
| `cargo run -- args` | ساخت + اجرا (⚠️ `--` جداکنندهٔ آرگومان‌های برنامه است) |
| `cargo test` | اجرای همهٔ تست‌ها + doctestها |
| `cargo test <name>` | فقط تست‌های هم‌نام با فیلتر |
| `cargo test -- --nocapture` | چاپ `println!`های تست‌ها را نشان بده |
| `cargo doc --open` | ساخت و باز کردن مستندات HTML |
| `cargo fmt` | قالب‌بندی خودکار |
| `cargo clippy -- -W clippy::pedantic` | لینت سخت‌گیرانه |
| `cargo tree` | درخت وابستگی‌ها |
| `cargo update` | به‌روزرسانی نسخه‌ها داخل قیدها |
| `cargo clean` | پاک کردن `target/` |
| `cargo add <crate>` | افزودن وابستگی (نیاز به `cargo-edit` در نسخه‌های قدیمی) |
| `cargo publish` | انتشار روی crates.io |

### فلگ‌های مفید
```bash
cargo build --offline            # بدون شبکه (کش موجود)
cargo build -j 4                 # محدود کردن موازی‌سازی
cargo build --timings            # گزارش کندی کامپایل (بهترین ابزار دیباگ سرعت!)
cargo test -- --test-threads=1   # اجرای سری تست‌ها (برای تست‌های IO)
cargo run --features foo         # فعال‌کردن feature
cargo run --example hello        # اجرای یک مثال از examples/
cargo bench                      # اجرای بنچمارک‌ها
```

### `Cargo.toml` — هر بخشی که لازم داری
```toml
[package]
name = "ar"
version = "0.1.0"            # SemVer: MAJOR.MINOR.PATCH
edition = "2024"             # نسخهٔ زبانی (نه نسخهٔ کامپایلر!)
rust-version = "1.85"        # MSRV — حداقل کامپایلر لازم
authors = ["..."]
description = "..."          # لازم برای publish
license = "MIT"
repository = "https://github.com/..."
keywords = ["cli", "rust"]   # حداکثر ۵
categories = ["command-line-utilities"]

[dependencies]               # وابستگی‌های زمان اجرا
serde = { version = "1", features = ["derive"], optional = true }

[dev-dependencies]           # فقط برای تست/مثال/بنچمارک
pretty_assertions = "1"

[build-dependencies]         # برای build.rs (کد تولید در زمان ساخت)
cc = "1"

[features]                   # قابلیت‌های اختیاری (مثل configure در C)
default = ["json"]
json = []
full = ["json", "extra"]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "unwind"             # "abort" = باینری کوچک‌تر، ولی catch_unwind کار نمی‌کند
strip = "debuginfo"

[profile.dev]
opt-level = 1                # ۰ = سریع‌ترین کامپایل، ۱ = تعادل خوب

[[bin]]                      # چند باینری در یک بسته
name = "ar"
path = "src/main.rs"

[[bench]]
name = "throughput"
harness = false

[workspace]                  # پروژه‌های چندبسته‌ای
members = ["cli", "core", "server"]
```

### 🎯 نسخه (=)، ~ و ^ — معنی قیدها
```toml
anyhow = "1.0.80"    # = ^1.0.80 → هر نسخهٔ 1.x با x ≥ 0.80 (سازگار)
anyhow = "^1.2"      # همان بالا (پیش‌فرض)
anyhow = "~1.2.3"    # >=1.2.3 و <1.3.0 (فقط patch)
anyhow = "=1.2.3"    # دقیقاً همین (خطرناک — جلوی وصله‌های امنیتی را می‌گیرد)
anyhow = "1.*"       # هر 1.x (بی‌قید)
```

### `Cargo.lock`
* برای **برنامه‌ها** (binary): کامیت کن ✅ (تا ساخت‌های تکراری یکسان باشند)
* برای **کتابخانه‌ها**: کامیت نکن ❌ (در `.gitignore` بگذار)

---

## ۳) ساختار استاندارد پروژه

```text
myproject/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── src/
│   ├── main.rs          # باینری (اختیاری)
│   └── lib.rs           # کتابخانه (اختیاری — هر دو را می‌توانی داشته باشی)
├── tests/               # تست‌های یکپارچگی (integration) — از بیرون، مثل کاربر
│   └── cli.rs
├── examples/            # مثال‌های اجرایی: cargo run --example <name>
│   └── basic.rs
├── benches/             # بنچمارک (نیاز به criterion یا #[bench] نایتلی)
│   └── speed.rs
├── build.rs             # کد ساخت (تولید کد، لینک کتابخانهٔ C، …)
└── target/              # خروجی‌ها (در .gitignore)
```

### الگوی «کتابخانه + باینری» (کاری که این پروژه می‌کند)
```rust
// src/lib.rs  →  منطق، تست‌پذیر و قابل استفاده در doctest
pub mod app;
pub mod json;

// src/main.rs →  لایهٔ نازک نمایش
use ar::app::App;

fn main() -> std::process::ExitCode { /* … */ }
```
چرا؟ چون **doctestها و تست‌های یکپارچگی نمی‌توانند باینری را import کنند.**

---

## ۴) چرخهٔ توسعه‌ای که واقعاً استفاده می‌شود

```bash
# ۱) ساخت و اجرا در حلقه
cargo run -- list

# ۲) پیش از هر کامیت
cargo fmt
cargo clippy --all-targets -- -D warnings   # هشدار را به خطا تبدیل کن
cargo test

# ۳) ساخت نهایی
cargo build --release
./target/release/ar demo all

# ۴) اندازهٔ باینری
ls -lh target/release/ar
cargo bloat --release      # با cargo-bloat: چه چیزی باینری را بزرگ کرده؟
```

### pre-commit hook ساده (فایل `.git/hooks/pre-commit`)
```bash
#!/bin/sh
set -e
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --quiet
```

---

## ۵) خالی‌کردن کش و حل مشکلات رایج

| مشکل | راه‌حل |
|------|-------|
| «cannot find package» بعد از افزودن وابستگی | `cargo update -p <crate>` یا `rm -rf ~/.cargo/registry` |
| ساخت گیر کرده | `cargo clean && cargo build` |
| خطای عجیب لینک | `cargo build -v` برای دیدن دستور لینک |
| کامپایل خیلی کند | `cargo build --timings` → معمولاً LTO یا `codegen-units=1` در `[profile.dev]` است |
| `error: linker not found` (لینوکس) | `sudo apt install build-essential` |
| نسخهٔ اشتباه rustc | `rustup override set stable` در همان پوشه |
| می‌خواهم بررسی‌های `unsafe` را بدانم | `cargo +nightly miri test` |

---

## ۶) env و متغیرهای محیطی مفید

```bash
RUST_BACKTRACE=1 cargo run     # پشتهٔ خطا هنگام panic
RUST_LOG=debug cargo run       # برای crateهایی که از tracing/log استفاده می‌کنند
CARGO_TARGET_DIR=/tmp/build cargo build   # پوشهٔ خروجی متفاوت (خوب برای CI)
RUSTFLAGS="-C target-cpu=native" cargo build --release  # بهینه‌سازی برای CPU خودت
NO_COLOR=1 cargo run           # خروجی بی‌رنگ (استاندارد no-color.org)
```

💡 در کد ما `env!("CARGO_PKG_VERSION")` و `option_env!("RUSTC_VERSION")` را
دیدید — این‌ها **در زمان کامپایل** جای‌گذاری می‌شوند، نه در اجرا.

[⬅️ قبلی: نقشهٔ راه](00-roadmap.md) · [بعدی: سینتکس ➡️](02-syntax.md)
