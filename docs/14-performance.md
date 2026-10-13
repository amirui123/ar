# 🚀 ۱۴ — کارایی و بهینه‌سازی

> **قانون صفر: اندازه‌گیری قبل از بهینه‌سازی.** ۹۰٪ «بهینه‌سازی‌های حس‌شده»
> بی‌اثرند یا مضرند.

---

## ۱) چرخهٔ کار

```bash
# ۱) اول بفهم کجا کند است
cargo build --release
perf stat ./target/release/ar demo all            # شمارش دستورات، cache-miss
perf record -g ./target/release/ar demo all && perf report
# یا در macOS: instruments / dtrace؛ در ویندوز: VS Profiler

# ۲) بنچمارک تکرارپذیر
cargo install hyperfine
hyperfine --warmup 3 './target/release/ar list --status all'

# ۳) بنچمارک ریز با criterion (Rust-native)
```

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
[[bench]]
name = "store_bench"
harness = false
```

```rust
// benches/store_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
fn bench(c: &mut Criterion) {
    c.bench_function("save 10k", |b| b.iter(|| { /* کد */ }));
}
criterion_group!(benches, bench);
criterion_main!(benches);
```
```bash
cargo bench        # + گزارش HTML در target/criterion/
```
🎯 `black_box(x)` داخل بنچمارک واجب است، وگرنه بهینه‌ساز کل کار را حذف می‌کند.

---

## ۲) پروفایل `release` — حرفه‌ای

```toml
[profile.release]
opt-level = 3
lto = "fat"            # ادغام بین crateها → درونخطی بیشتر
codegen-units = 1      # بهینه‌سازی بین واحدها
panic = "abort"        # باینری کوچک‌تر، سرریز سریع‌تر
strip = true           # حذف نمادهای دیباگ
incremental = false
debug = false

[profile.bench]
lto = "fat"            # بنچمارک باید مثل production باشد

[profile.dev]
opt-level = 0          # سریعترین کامپایل، کندترین اجرا

# پروفایل «کند کامپایل، سریع اجرا» برای تستها
[profile.dev.package."*"]
opt-level = 2          # وابستگیها بهینه، کد خودت دیباگپذیر
```
⚠️ `panic = "abort"` با `cargo test` سازگار نیست (برای پروفایل release جداست).

---

## ۳) تخصیص‌ها = دشمن شمارهٔ یک

```rust
// ❌ در حلقه: ۱۰۰۰ تخصیص
for line in lines { out = out + line + "\n"; }

// ✅ e یک بار
let mut out = String::with_capacity(lines.len() * 32);
for line in lines { out.push_str(line); out.push('\n'); }

// ✅ یا
let out: String = lines.iter().map(|l| format!("{l}\n")).collect();

// ❌ cloneهای ضمنی
for t in &tasks { process(t.clone()); }     // هر بار!
// ✅
for t in &tasks { process_ref(t); }          // بگیر &Task
```

**ابزار:**
```bash
# شمارش تخصیصها با allocator شمارنده
valgrind --tool=massif ./target/release/ar list   # (Linux)
heaptrack ./target/release/ar list                 # گرافیکی
```
(در Rust، الگوی رایج: یک `GlobalAlloc` شمارنده در تست‌ها.)

---

## ۴) ساختار دادهٔ درست

| کار | انتخاب | چرا |
|-----|--------|-----|
| جست‌وجوی کلیدی | `HashMap` / `BTreeMap` | O(1) / O(log n) مرتب |
| مجموعهٔ کوچک (زیر ۱۶) | `Vec` + `contains` | حافظهٔ پیوسته، سریع‌تر از هش |
| مجموعهٔ بزرگ فقط‌عضویت | `HashSet` / `BTreeSet` | |
| صف دو سر | `VecDeque` | نه `Vec::remove(0)` (O(n)) |
| صف اولویت | `BinaryHeap` | O(log n) |
| کلید ترتیبی با ترتیب درج | `IndexMap` (crate) | |
| شمارنده پیوسته | `Vec<u32>` با اندیس = شناسه | بدون هش |
| رشتهٔ کوتاه پرتکرار | `Rc<str>` / `Arc<str>` / interner | |
| دادهٔ هم‌بعد | `ndarray`/`nalgebra` | SIMD-پسند |

🎯 **`HashMap` پیش‌فرض `SipHash` است** — امن در برابر حملهٔ DoS ولی کند.
اگر ورودی از کاربر نیست و سرعت مهم است: `hashbrown` یا `ahash` یا `FxHashMap`
(`rustc-hash`):
```rust
use rustc_hash::FxHashMap;
let mut m = FxHashMap::default();
```

---

## ۵) بهینه‌سازی‌های سطح کد

```rust
// ۱) بردار پیش‌تخصیص
let mut v = Vec::with_capacity(n);

// ۲) entry بهجای دو جستوجو
*counts.entry(k).or_insert(0) += 1;

// ۳) extend بهجای push در حلقه
v.extend(iter);

// ۴) sort_unstable برای انواع بدون ترتیب مساوی مهم
v.sort_unstable();            // سریعتر، حافظهٔ کمتر

// ۵) select_nth_unstable برای «kامین» بدون کامل سورتکردن
let (_, kth, _) = v.select_nth_unstable(n / 2);

// ۶) بافر خواندن فایل
use std::io::BufReader;
let f = BufReader::new(std::fs::File::open("big")?);   // 🎯 بدون این، هر خواندن = syscall
// نوشتن:
use std::io::BufWriter;
let mut w = BufWriter::new(std::fs::File::create("out")?);
w.write_all(b"...")?;         // و در پایان flush (Drop خودش میکند)

// ۷) قفل را از حلقه بیرون بکش
let mut g = m.lock().unwrap();
for x in data { g.insert(x); }

// ۸) رشتهها: با_str / &str در امضاها نه String
// ۹) #[inline] فقط برای توابع ریز و پرتکرار (و بعد از پروفایل)
// ۱۰) دورهای کوچک را unroll نکن؛ LLVM خودش میکند
```

---

## ۶) موازی‌سازی (ساده‌ترین برد بزرگ)

```rust
// rayon — سه خط برای موازی
use rayon::prelude::*;
let sum: u64 = data.par_iter().map(|x| x * 2).sum();
data.par_iter().for_each(|x| expensive(x));

// chunk و map بدون rayon (std)
let handles: Vec<_> = chunks.into_iter().map(|c| thread::spawn(move || work(c))).collect();
let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
```
🎯 قوانین: کار باید مستقل و به‌قدر کافی سنگین باشد (بالای ~۱۰ میکروثانیه)،
وگرنه سربارهٔ هم‌زمانی از سود بیشتر می‌شود.

---

## ۷) IO

| ترفند | چرا |
|-------|-----|
| `BufReader`/`BufWriter` | کاهش syscall |
| `read_to_string` یکجا برای فایل کوچک | ساده‌ترین و اغلب سریع‌ترین |
| `write_all` به‌جای `write` | `write` می‌تواند ناقص بنویسد |
| `File::create` قبل از `rename` | اتمیک (همان الگوی `FileStore`) |
| `memmap2` برای فایل‌های بزرگ فقط‌خواندنی | بدون خواندن صریح |
| `TcpStream::set_nodelay(true)` | کاهش تأخیر در اتصال‌های کوچک |
| `BytesMut` (crate) برای پروتکل‌ها | بدون کپی |

---

## ۸) باینری کوچک و بار سریع

```toml
[dependencies]
# صفر! (پروژهٔ ما: فقط std) → 40-60% کاهش حجم
```
```bash
cargo build --release
ls -lh target/release/ar          # بدون strip ~۴MB، با strip ~400KB
cargo bloat --release --crates    # چه چیزی حجم را میخورد
# nightly:
cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu
```
سایر ترفندها: `strip = true` · `panic = "abort"` · اجتناب از `format!` در
مسیرهای سرد · `opt-level = "z"` (کِی حجم مهم است) · حذف `serde` اگر لازم نیست.

---

## ۹) زمان کامپایل

| ترفند | سود |
|-------|-----|
| `[profile.dev.package."*"] opt-level = 2` | اجرای سریع، کامپایل وابستگی یک‌بار |
| `sccache` یا `mold`/`lld` | ۲–۵ برابر سریع‌تر |
| `cargo check` به‌جای `build` در حلقهٔ توسعه | ۳–۱۰ برابر |
| حذف `serde`/`clap` از مسیرهای سرد | بزرگ |
| featureهای محدود (`tokio` با featureهای دقیق) | بزرگ |
| `RUSTFLAGS="-C link-arg=-fuse-ld=lld"` | لینک سریع |
| تقسیم به workspace و crateهای کوچک | کامپایل موازی |

---

## ۱۰) جدول «از کجا شروع کنم؟»

| نشانه | حدس اول |
|-------|---------|
| CPU بالا، IO کم | الگوریتم/تخصیص |
| IO بالا، CPU کم | بافر، batch، فایل‌های بزرگ |
| کندی با دادهٔ بزرگ ولی سریع با کوچک | پیچیدگی الگوریتمی (O(n²)) |
| کندی فقط در debug | نرمال است؛ با `--release` بسنج |
| کندی در نسخهٔ موازی | رقابت قفل / تعداد ترد زیاد |
| مصرف حافظهٔ بالا | `clone` اضافه، `Vec` بدون `shrink_to_fit` |
| بار اول کند، بعد سریع | cache/ترتیب دهی |
| بار اول سریع، بعد کند | نشت حافظه / رشد ساختار |

---

## ۱۱) ترفندهای ریز که همیشه کمک می‌کنند

```rust
// ۱) از alloc برای هر عملیات نمایش استفاده نکن
use std::fmt::Write;
write!(&mut buf, "{x}")?;                  // بدون String جدید

// ۲) پیام خطا را تنبل بساز
if cond { return Err(AppError::Usage(format!("..."))); }   // ✅ فقط در مسیر خطا

// ۳) از &'static str برای پیامهای ثابت استفاده کن (بدون تخصیص)
// ۴) برای شمارش از fold بیهوده استفاده نکن: `iter.count()`
// ۵) در سرور: پاسخهای کوچک را یکجا بنویس (یک syscall)
// ۶) در سرور: keep-alive را پیاده کن (اتصال مجدد گران است)
// ۷) `Instant` برای اندازهگیری (نه SystemTime) — یکنوا
```

```rust
let t = std::time::Instant::now();
// … کار …
eprintln!("⏱ {:?}", t.elapsed());
```

---

> 📚 منابع: *Rust Performance Book* (nnethercote.github.io/perf-book) ·
> *Rust for Rustaceans* فصل ۷ · `cargo bench` + criterion · perf/valgrind.

[⬅️ قبلی: دام‌ها](13-pitfalls.md) · [بعدی: چیت‌شیت std ➡️](15-std-cheatsheet.md)
