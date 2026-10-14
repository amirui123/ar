# 🧨 ۱۱ — `unsafe` و FFI

## ۱) قاعدهٔ اول

> `unsafe` یعنی: **کامپایلر نمی‌تواند بررسی کند، پس تو تضمین می‌کنی.**
> اگر تضمینت غلط باشد، نتیجه «رفتار تعریف‌نشده» (UB) است — که هیچ تضمینی
> ندارد: ممکن است کار کند، ممکن است فردا در release بشکند.

`unsafe` چهار قدرت می‌دهد:

| قدرت | خطر |
|------|-----|
| `*const T` / `*mut T` را deref کنی | اشاره‌گر معلق/باطل = UB |
| `unsafe fn` را صدا بزنی | قراردادش را باید رعایت کنی |
| `static mut` را بخوانی/بنویسی | داده‌ی رقابتی |
| `union`/`transmute` | نقض نوع |

**`unsafe` این‌ها را خاموش نمی‌کند:** بررسی قرض، بررسی نوع، پنیک، بدهی
حافظه — همه سر جایشان هستند.

---

## ۲) اشاره‌گر خام در عمل

```rust
fn main() {
    let v = vec![10, 20, 30];
    let p: *const i32 = v.as_ptr();          // اشاره‌گر خام از یک مرجع معتبر

    // ✅ امن چون داخل مرزهای برش است و v زنده است
    let second = unsafe { *p.add(1) };       // add(1) = یک عنصر جلو
    println!("{second}");                    // 20

    // ⚠️ الگوی «خواندن بدون بررسی مرز» (کاری که دمو unsafe_ffi ما نشان می‌دهد)
    let idx = 2;
    let fast = unsafe { *p.add(idx) };       // ← بدون bound check از خودِ slice سریع‌تر
    println!("{fast}");                      // 30
}
```

🎯 **کاربرد واقعی این ترفند:** در مسیرهای داغی که قبلاً ثابت کرده‌ای ایندکس
حتماً در مرز است (مثلاً بعد از یک `if idx < len`)، می‌توانی یک بار بررسی مرز
را حذف کنی. سود آن معمولاً **کم و ریسکش زیاد** است — اول با بنچمارک ثابت کن
که گلوگاه همان است.

### قرارداد نوشتن `unsafe fn` — همیشه `# Safety`
```rust
/// # Safety
/// `idx` باید کوچک‌تر از `self.len()` باشد.
pub unsafe fn read_unchecked(self, idx: usize) -> Option<i32> {
    if idx >= self.len() { return None; }        //
    Some(unsafe { *self.as_ptr().add(idx) })
}

// ← ترفند مهم: یک پوشش (wrapper) امن روی منطق ناامن
pub fn safe_get(self, idx: usize) -> i32 {
    // تنها جایی که قرارداد را میشکنیم اینجاست، و همینجا هم بررسی میکنیم
    unsafe { self.read_unchecked(idx) }.unwrap_or(-1)
}
```
📌 قاعدهٔ طلایی: **`unsafe` را کوچک، ایزوله و مستند کن**؛ بقیهٔ کد باید
`safe` بماند.

---

## ۳) تفاوت `*const T` و مرجع

| | مرجع `&T` | اشاره‌گر خام `*const T` |
|---|---|---|
| تهی (`null`) | ✅ ندارد | ممکن است |
| معلق | ❌ (زمان کامپایل) | ممکن است |
| هم‌ترازی | تضمین‌شده | تضمین‌نشده |
| بررسی قرض | ✅ | ❌ |
| `Send`/`Sync` | خودکار | خودکار نیست |
| کجا | کد روزمره | داخل ماژول‌های ایزوله |

---

## ۴) `transmute` — آخرین راه

```rust
// خواندن یک f32 از بایتهایش (روش درست و امن):
let f: f32 = f32::from_bits(0x4048_F5C3);        // ✅
// یا از بایت:
let f2 = f32::from_le_bytes([0x00, 0x00, 0x80, 0x3f]);   // ✅ (1.0)

// transmute (اینطور نکن تا مجبور نشوی):
// let bad: u32 = unsafe { std::mem::transmute(1.0f32) };   // همین کار را میکند اما...
```
✅ جانشین‌های امن `transmute`:
| هدف | راه امن |
|------|---------|
| بیت‌های f32↔u32 | `f32::to_bits` / `from_bits` |
| بایت←عدد | `u32::from_le_bytes` و خانواده‌اش |
| کلاس C enum | `#[repr(u8)]` + `TryFrom` |
| اشاره‌گر↔usize | `as` (استفاده از `ptr::addr`) |
| `&mut [T]` | `split_at_mut` |

⚠️ `transmute` بین طول‌های مختلف اصلاً کامپایل نمی‌شود؛ بین انواع
هم‌اندازه می‌شود و می‌تواند UB باشد.

---

## ۵) `static mut` و `union` — دام‌ها

```rust
// ❌ ممنوع در edition 2024 بدون unsafe، و به هر حال خطرناک
// static mut COUNTER: u32 = 0;

// ✅ جایش:
use std::sync::atomic::{AtomicU32, Ordering};
static COUNTER: AtomicU32 = AtomicU32::new(0);

// ✅ یا:
use std::cell::UnsafeCell;            // پایهٔ همهٔ سلول‌های تعدیل‌پذیر
struct OneWriter { inner: UnsafeCell<u32> }
unsafe impl Sync for OneWriter {}     // ← مسئولیت تویی
```

`union` (مثل C) برای تعبیر بایت‌های یک مقدار به شکل دیگر — خواندن از آن
`unsafe` است و باید بدانی کدام عضو فعال است.

---

## ۶) FFI — حرف‌زدن با C

### فراخوانی کد C از Rust

```rust
use std::ffi::{c_char, c_int, CStr, CString};

unsafe extern "C" {                      // syntax جدید (edition 2024)
    fn strlen(s: *const c_char) -> usize;
    fn puts(s: *const c_char) -> c_int;
}

fn main() {
    let text = CString::new("سلام از C").unwrap();   // ← حتماً با NUL داخلی چک میکند
    let len = unsafe { strlen(text.as_ptr()) };
    println!("طول: {len}");
}
```
📌 نکته‌ها:
* `CString` نمی‌گذارد بایت `\0` داخل رشته باشد (می‌دهد `NulError`) — چون C
  رشته را تا NUL می‌خواند.
* `CStr::from_ptr` برای گرفتن رشته از C (و `to_str()` که UTF-8 چک می‌کند).
* انواع C: `c_int`, `c_char`, `c_void`, `size_t` (از `libc` یا `std::ffi`).

### ماژول امن دور FFI (الگوی استاندارد)
```rust
pub struct CString2 { inner: CString }

impl CString2 {
    pub fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        Ok(Self { inner: CString::new(s)? })
    }
    pub fn len(&self) -> usize {
        unsafe { strlen(self.inner.as_ptr()) }     // ← unsafe فقط همینجا
    }
}
```
🚩 این «مرز امنیت» است: دنیای بیرون همیشه `safe` می‌بیند.

### اکسپورت تابع Rust به C
```rust
#[unsafe(no_mangle)]                       // در edition 2024 نیاز به unsafe دارد
pub extern "C" fn ar_add(a: i32, b: i32) -> i32 { a + b }
```

### ابزارها
```bash
cargo install cbindgen    # ساخت .h از کد Rust
cargo install bindgen-cli # ساخت binding از .h به Rust
cargo miri test           # اجرای تستها در مفسر Miri و شکار UB
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test   # ASan
```

---

## ۷) `unsafe` در جاهای غیرمنتظره

`unsafe` فقط برای اشاره‌گر نیست:

| جا | چرا |
|----|-----|
| `Vec::get_unchecked` | حذف بررسی مرز |
| `str::from_utf8_unchecked` | وقتی مطمئنی UTF-8 است (مثلاً از خودِ str بریده‌ای) |
| `slice::from_raw_parts` | ساخت برش از دادهٔ خام |
| `ManuallyDrop` | جلوگیری از اجرای Drop (FFI، union) |
| `ptr::read`/`write` | بدون اجرای دِراپ |
| trait با متد نامنبعی | `unsafe trait` برای قواعدی که کامپایلر چک نمی‌کند (`Send`/`Sync` دستی) |
| `MaybeUninit` | مقداردهی جزئی آرایه‌ها |

---

## ۸) چک‌لیست `unsafe` حرفه‌ای

1. **دلیلش را بنویس:** یک کامنت `// SAFETY:` بالای هر بلوک، دقیقاً بگو چرا
   شرایط برقرار است.
2. **کوچکش کن:** تا جای ممکن کد unsafe در یک تابع کوچک و ایزوله.
3. **مرز امن بساز:** API بیرونی نباید `unsafe` باشد (اگر می‌توانی).
4. **با Miri تست کن:** `cargo +nightly miri test` — بسیاری از UBها را می‌گیرد.
5. **مرزها را دوباره چک کن:** هر بار که داده از بیرون می‌آید.
   ```rust
   debug_assert!(idx < self.len());        // در توسعه چک کن، در release نخواه
   ```
6. **کم استفاده کن:** بیشتر UBها از کدی می‌آید که لازم نبود unsafe باشد.
7. **مستند کن:** هر پیش‌شرط را در بخش `# Safety` بنویس — هم برای خودت، هم
   برای `rustdoc`.

---

## ۹) چه کاری را با `unsafe` **نگیر**؟

| نکن | به‌جایش |
|-----|---------|
| `unsafe` برای سرعت حدس‌زده | پروفایل کن؛ ممکن است بهینه‌ساز خودش بهتر بدهد |
| `transmute` بین structها | `From`/`TryFrom` بنویس |
| `static mut` برای وضعیت سراسری | `AtomicX` یا `OnceLock`/`Mutex` |
| `get_unchecked` بدون اثبات | `get()` — هزینه‌اش ناچیز است |
| `unsafe impl Send` بدون دلیل | ساختار را واقعاً Sendپذیر کن |
| raw pointer برای رد کردن borrow checker | طراحی را عوض کن (شاخص، `Rc`, `RefCell`) |

🎯 **جملهٔ آخر:** در Rust، `unsafe` نشانِ بلوغ است، نه راه فرار. کسی که
`unsafe` می‌نویسد باید بتواند *دقیقاً* بگوید کدام قرارداد برقرار است و چرا.

[⬅️ قبلی: ماکروها](10-macros.md) · [بعدی: ۱۰۰ ترفند ➡️](12-tricks.md)
