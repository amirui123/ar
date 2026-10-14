# 🕳️ ۱۳ — دام‌ها و اشتباه‌های رایج

> هر مورد: نشانه → علت → درمان. این‌ها همان چیزهایی هستند که وقت آدم را می‌خورند.

---

## ۱) عدد صحیح

| نشانه | علت | درمان |
|-------|-----|-------|
| پنیک `attempt to subtract with overflow` فقط در debug | در debug سرریز **پنیک** می‌کند، در release **دور می‌زند** | `checked_*`, `saturating_*`, `wrapping_*`, `overflowing_*` |
| `panicked at 'attempt to negate with overflow'` | `i64::MIN.abs()` | `unsigned_abs()` |
| تقسیم بر صفر | `10 / 0` | `checked_div` |
| `as` بی‌سروصدا داده را می‌بُرد | `300u32 as u8 == 44` | `u8::try_from(x).unwrap()` |

```rust
let x: u8 = 250;
println!("{} {}", x.wrapping_add(10), x.saturating_add(10));  // 4 255
println!("{:?}", u8::try_from(300u32));                       // Err(...)
```
🎯 قانون: **`as` فقط بین انواعی که مطمئنی بی‌خطر است** (مثل `u8 → u32`).

---

## ۲) رشته‌ها (بلای متن فارسی)

| نشانه | علت | درمان |
|-------|-----|-------|
| `byte index N is not a char boundary` | `&s[i..j]` وسط کاراکتر چندبایتی | `s.get(i..j)`, `is_char_boundary`, `char_indices` |
| `s.len()` عدد بزرگ‌تر از انتظار | طول **بایت** است نه کاراکتر (فارسی ۲ بایت) | `s.chars().count()` |
| برعکس‌شدن/جابه‌جایی حروف | برش بایتی وسط نویسهٔ مرکب | کار در سطح `chars()` |
| `s[0]` کار نمی‌کند | `str` اندیس‌پذیر نیست | `s.chars().next()` |
| `\r\n` اضافه در ورودی | فایل‌های ویندوزی | `trim()` / `trim_end_matches(['\r','\n'])` |

```rust
let s = "سلام دنیا";
println!("{} {}", s.len(), s.chars().count());  // 17 9
println!("{:?}", s.get(0..1));                  // None (چون وسط «س» است)
```

---

## ۳) قرض (borrow)

| خطا | علت | درمان |
|-----|-----|-------|
| `cannot borrow as mutable because it is also borrowed as immutable` | استفاده هم‌زمان | محدوده را کوچک کن، `clone` کن، یا بازچینش کن |
| `does not live long enough` | مقدار محلی زودتر از مرجعش مرده | مالکیت بده (`String` نه `&str`) |
| `cannot move out of … which is behind a shared reference` | فقط `&T` داری | `clone`, `Copy`, یا `mem::take` |
| `value borrowed here after move` | انتقال و استفاده دوباره | `clone` یا `&` دادن |
| `cannot return reference to local` | برگرداندن `&` به محلی | برگردان مالک (`String`/`Vec`) |
| `cannot borrow *x as mutable, as it is behind &` | در حلقه روی `&` تکرار می‌کنی | `for x in &mut v` |

🎯 **ترتیب درمان‌ها:** طراحی → محدودهٔ کوچک‌تر → `&` → `clone` (آخرین راه؛ نه
چیزی که از اول می‌زنی).

---

## ۴) پنیک‌های غافلگیرکننده

| پنیک | علت |
|------|-----|
| `index out of bounds` | `v[i]` بدون بررسی (کارِ `v.get(i)`) |
| `called Option::unwrap() on a None value` | `unwrap` روی نبودِ ارزش |
| `called Result::unwrap() on an Err value` | `unwrap` روی خطا |
| `slice index starts at 2 but ends at 1` | `&v[2..1]` |
| `the len is 3 but the index is 5` | `split_at` یا `remove` خارج از مرز |
| `lock()` بعد از پنیک | قفل مسموم شده |
| `RefCell already borrowed` | `borrow_mut` هم‌زمان |
| `capacity overflow` | ساختار غول با تخصیص |
| `attempt to divide by zero` | تقسیم |
| `called String::from_utf8 … invalid utf-8` | دادهٔ غیر UTF-8 |

```rust
// 🎯 جای unwrap: مقدار پیش‌فرض یا نتیجه‌یابی
let x = opt.unwrap_or(0);
let y = opt.unwrap_or_else(|| expensive());
let z = res.unwrap_or_default();
let w = res.expect("فایل تنظیمات باید موجود باشد");
```

---

## ۵) کامپایلر سختگیر است — ولی این‌ها را نمی‌گیرد

| دام | چرا خطرناک |
|-----|-----------|
| `Ordering::Relaxed` بی‌جا | داده‌ی قدیمی |
| `RefCell` بیش‌ازحد | پنیک زمان اجرا (قواعد قرض در runtime) |
| حلقه‌های بی‌پایان بدون `sleep` | مصرف ۱۰۰٪ CPU |
| `loop { rx.recv() }` بدون `drop(tx)` | انتظار ابدی |
| `clone` در مسیر داغ | کندی بی‌دلیل |
| `unwrap` در کتابخانه | پنیک در دست کاربر |
| `static mut` | داده‌ی رقابتی |
| حافظه‌ای‌شدن با `Rc` چرخه‌ای | نشتی حافظه (Rust نشت **دارد**!) |
| `unsafe` بی‌مستند | UB |
| `HashMap` در معرض حملهٔ hash | DoS (باید hasher تصادفی باشد — پیش‌فرض هست) |

> ⚠️ **Rust نشت حافظه را ممنوع نمی‌کند** — `Box::leak`, `mem::forget` و چرخهٔ
> `Rc` همه ممکن‌اند. Rust «امنیت حافظهٔ بدون UB» را تضمین می‌کند، نه «عدم نشت».

---

## ۶) async

| دام | نشانه | درمان |
|-----|-------|-------|
| `async fn` صدا زدی و `.await` نزدی | هیچ اتفاقی نمی‌افتد | `.await` یا `spawn` |
| `std::sync::MutexGuard` بین `.await` | کامپایل نمی‌شود (ممنوع!) | `tokio::sync::Mutex` |
| کار CPU در task | کل runtime می‌خوابد | `spawn_blocking` |
| لغو task وسط کار | وضعیت نیمه‌کاره | cancel-safe بنویس |
| `block_on` داخل async | بن‌بست | `spawn` |
| نبود `wake()` در executor خودت | کار معلق ابدی | `Waker` را ذخیره و صدا کن |

---

## ۷) کار با فایل و مسیر

| دام | درمان |
|-----|-------|
| مسیرهای ویندوزی `\` | `PathBuf::join` به‌جای `format!("{a}/{b}")` |
| `read_to_string` روی فایل غیر UTF-8 | `read` + `String::from_utf8_lossy` |
| نوشتن نیمه‌کاره (crash وسط نوشتن) | «فایل موقت + rename» (اتمیک) — همان کاری که `FileStore` می‌کند: نوشتن در `tasks.json.tmp` بعد `fs::rename` |
| بازنکردن فایل با `\n` در نام | مسیر را همیشه `Path` بگیر |
| `canonicalize` روی فایل ناموجود | خطا می‌دهد؛ با `if p.exists()` چک کن |

---

## ۸) زبانه/امضا در برابر نقشهٔ ذهنی

| نشانه | علت |
|-------|-----|
| `expected &str, found String` | فقط `&` بده یا `as_str()` |
| `no method named to_string found for &T` | trait در دامنه نیست → `use std::string::ToString;` (یا `format!`) |
| `the trait bound T: Display is not satisfied` | نوع با `{}` چاپ نمی‌شود → `{:?}` یا `#[derive(Debug)]` |
| `cannot compare &str with String` | `a.as_str() == b.as_str()` |
| `cannot infer type` | تایپ را صریح بده: `let v: Vec<i32> = …collect();` |
| `use of moved value` در حلقه | `for x in v` را `for x in &v` کن |
| `expected (), found i32` | `;` اضافه یا کم |
| `type annotations needed` در `collect` | `collect::<Vec<_>>()` |

---

## ۹) تست

| دام | توضیح |
|-----|-------|
| تست‌ها موازی‌اند | وضعیت سراسری مشترک → وابستگی ترتیب |
| `assert_eq` روی float | `(a - b).abs() < 1e-9` |
| `assert_eq!(format!("{}", 2), "۲")` | `{}` رقم لاتین می‌دهد (خودمان گرفتارش شدیم!) |
| `#[should_panic]` بدون `expected` | هر پنیکی را قبول می‌کند |
| کد تست در باینری | از `#[cfg(test)] mod tests` استفاده کن |
| نبودن تست برای حالت‌های مرزی | خالی، تک‌عضوی، `i64::MIN`, رشتهٔ فارسی |

---

## ۱۰) تله‌های ظریف

```rust
// ۱) if let با قفل — نگهبان تا پایان بلوک میماند
// ۲) `while let` روی کانال → تا بستهشدن همه فرستندهها
// ۳) `&s[..]` روی رشتهٔ فارسی
// ۴) float منفی صفر: -0.0 == 0.0 ولی 1.0 / -0.0 = -inf
// ۵) NaN != NaN  → برای sort از partial_cmp استفاده نکن
v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
// ۶) `a < b` روی f64 با NaN همیشه false
// ۷) جمع float ترتیبپذیر نیست → دقت تغییر میکند
// ۸) `Duration::from_millis(1)` روی بلوک شبیهسازی میشود (`div_duration_f64`)
// ۹) `HashMap` ترتیب ندارد → برای خروجی پایدار، کلیدها را sort کن
let mut items: Vec<_> = map.iter().collect();
items.sort_by_key(|(k, _)| *k);
// ۱۰) `char` ۴ بایتی است؛ `[u8; 4]` = char نیست → from_u32
// ۱۱) `#[derive(PartialOrd)]` ترتیب فیلدها را رعایت میکند (نه ترتیب نوشتاری)
// ۱۲) env vars را نتوانی در زمان اجرا تغییر بدهی → OK، هر ترد جدا
```

---

## ۱۱) حلقه‌های تکراری که به بن‌بست می‌خورند

1. **`clone` بزن تا کامپایل شود** → بعداً کارایی خراب می‌شود.
   ✅ طراحی را بفهم (مالکیت/عمر) و درستش کن.
2. **`unsafe` به‌جای بازطراحی** → UB دور یا نزدیک.
3. **همه چیز در `main.rs`** → بعد از ۵۰۰ خط، هر تغییری شجاعت می‌خواهد.
   ✅ دقیقاً الگوی `cli/store/app/model/…` این پروژه.
4. **بی‌توجهی به هشدارها** → `cargo build` باید صفر هشدار بدهد.
5. **`Result` را با `unwrap` خفه‌کردن در لایهٔ پایین** → خطای کاربر نمی‌فهمد
   چه شد.

---

## ۱۲) چک‌لیست نهایی پیش از هر کامیت

```bash
cargo fmt                                       # فرمت
cargo clippy --all-targets -- -D warnings       # صفر هشدار
cargo test                                      # همه سبز
cargo build --release                           # نسخهٔ واقعی
cargo run -- demo all                           # اجرای واقعی
cargo doc --no-deps                             # مستندات خراب نباشد
```
و این سؤال‌ها:
- [ ] هر `unwrap` دلیل دارد؟
- [ ] خطاها به فارسی و کاربرپسندند؟
- [ ] کد `unsafe` کامنت `// SAFETY:` دارد؟
- [ ] داکیومنت‌ها با واقعیت کد یکی‌اند (تعداد تست‌ها، نام ماژول‌ها)؟
- [ ] حالت‌های خالی/مرزی تست شده‌اند؟

[⬅️ قبلی: ۱۰۰ ترفند](12-tricks.md) · [بعدی: کارایی ➡️](14-performance.md)
