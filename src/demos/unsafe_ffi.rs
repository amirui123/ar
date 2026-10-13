//! # دموی `unsafe` و FFI — جایی که قواعد را خودت تضمین می‌کنی
//!
//! 🎯 **`unsafe` جادو نیست؛ فقط «پنج کار» را باز می‌کند:**
//! 1. درآیند کردن اشاره‌گر خام (`*const T` / `*mut T`)
//! 2. صدا زدن تابع `unsafe` / `extern`
//! 3. دسترسی یا تغییر `static mut`
//! 4. پیاده‌سازی صفت `unsafe` (مثل `Send`/`Sync`)
//! 5. دسترسی به فیلد `union`
//!
//! **هیچ‌کدام از این‌ها بررسی‌های عادی را «خاموش» نمی‌کند** — بقیهٔ کد باز هم چک می‌شود.
//!
//! 📜 قاعدهٔ طلایی: هر بلوک `unsafe` باید یک کامنت `SAFETY:` داشته باشد که
//! بگوید *چرا* شرایط برقرار است. در کد حرفه‌ای این اجباری است.

use std::ffi::{c_char, c_int};

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) اشاره‌گر خام (raw pointer)"));
    raw_pointers();

    println!("\n{}", color::bold("۲) صدا زدن C — تابع abs"));
    ffi_call();

    println!("\n{}", color::bold("۳) ساختار مشترک با C (`#[repr(C)]`)"));
    repr_c();

    println!("\n{}", color::bold("۴) توابع unsafe با قرارداد"));
    unsafe_fns();

    println!("\n{}", color::bold("۵) مالکیت دستی حافظه (Box::into_raw / from_raw)"));
    manual_memory();

    println!("\n{}", color::bold("۶) فهرست بررسی ایمنی"));
    checklist();
}

// ═══════════════════════════════════════════════════════════════════════════
fn raw_pointers() {
    let value: i32 = 42;

    // ساخت اشاره‌گر خام از یک مرجع — این کار *خودش* unsafe نیست،
    // ولی تضمینی هم دربارهٔ اعتبارش نمی‌دهد (مثل ارجاع گرفتن آدرس در C).
    let ptr: *const i32 = &value;

    println!("  آدرس اشاره‌گر: {:p}", ptr);
    println!("  آیا تهی است؟ {}", ptr.is_null());

    // ✅ درآیند کردن نیاز به unsafe دارد. قرارداد ایمنی:
    //    SAFETY: ptr از یک مرجع معتبر ساخته شده و تا اینجا زنده است.
    let deref = unsafe { *ptr };
    println!("  مقدار از طریق اشاره‌گر: {deref}");
    assert_eq!(deref, 42);

    // اشاره‌گر قابل تغییر — می‌توانیم از طریق آن بنویسیم (اینجا روی یک کپی)
    let mut data = [10i32, 20, 30];
    let mut_ptr: *mut i32 = data.as_mut_ptr();

    // SAFETY: ایندکس ۱ داخل محدودهٔ آرایه است و اشاره‌گر معتبر.
    unsafe {
        *mut_ptr.add(1) = 99; // حساب اشاره‌گر (pointer arithmetic)
    }
    println!("  بعد از نوشتن از طریق اشاره‌گر: {data:?}");

    // خواندن با offset
    // SAFETY: هر سه ایندکس داخل محدوده‌اند.
    let second = unsafe { mut_ptr.add(1).read() };
    println!("  خواندن عنصر ۱ با read(): {second}");

    // نسخهٔ امن همین کار (که همیشه باید ترجیح داده شود!):
    data[2] = 77;
    println!("  نسخهٔ امن (بدون unsafe): {data:?}");

    // 🔑 ترفند حرفه‌ای: اشاره‌گر خام را به برش امن تبدیل کن، یک بار:
    let slice: &[i32] = {
        // SAFETY: اشاره‌گر به یک آرایهٔ زندهٔ ۳ عنصری اشاره می‌کند.
        unsafe { std::slice::from_raw_parts(data.as_ptr(), data.len()) }
    };
    println!("  تبدیل به برش امن: {slice:?} (از این به بعد بدون unsafe)");
    println!("  📌 الگو: «unsafe را در کوچک‌ترین دامنه حبس کن، بیرونش API امن بده»");
}

// ═══════════════════════════════════════════════════════════════════════════
// در Rust 2024، بلوک‌های `extern` باید `unsafe` باشند (صریح‌سازی مرز خطر).
unsafe extern "C" {
    /// مقدار مطلق |i|
    fn abs(i: c_int) -> c_int;
    /// طول رشته تا نول
    fn strlen(s: *const c_char) -> usize;
    /// چاپ رشتهٔ C
    fn puts(s: *const c_char) -> c_int;
    /// شناسهٔ فرایند (فقط لینوکس/یونیکس)
    fn getpid() -> c_int;
}

fn ffi_call() {
    // SAFETY: abs همیشه امن است؛ هر i32 را می‌پذیرد.
    let m = unsafe { abs(-42) };
    println!("  abs(-42) از C = {m}");

    // رشتهٔ C درون‌خطی (Rust 1.77+): `c"..."`
    let greeting = c"سلام از زبان C!";
    println!("  طول رشته با strlen در C = {} بایت", unsafe { strlen(greeting.as_ptr()) });

    // SAFETY: اشاره‌گر به یک رشتهٔ C معتبر و نول‌ترمینال است.
    unsafe {
        puts(greeting.as_ptr());
    }

    // SAFETY: getpid هیچ پیش‌نیازی ندارد.
    println!("  شناسهٔ فرایند (getpid): {}", unsafe { getpid() });

    // ⚠️ خطرها و مسئولیت‌ها:
    //  • اگر رشتهٔ ما نول‌ترمینال نبود، `strlen` و `puts` تا بی‌نهایت می‌رفتند.
    //  • اگر تابع C حافظه را آزاد می‌کرد، باید می‌دانستیم کدام تخصیص‌دهنده.
    //  • هیچ‌کدام از این‌ها را کامپایلر Rust برایت چک نمی‌کند.
    println!("  ⚠️ مرز FFI: کامپایلر هیچ‌چیز دربارهٔ سمت C نمی‌داند — مسئولیت کامل با توست");
}

// ═══════════════════════════════════════════════════════════════════════════
/// ساختاری با چیدمان تضمین‌شده برای اشتراک با کد C.
///
/// 🔑 `#[repr(C)]` یعنی: فیلدها به ترتیب و با قواعد C در حافظه چیده می‌شوند.
/// بدون آن، Rust می‌تواند فیلدها را جابه‌جا/فشرده کند و کد C خراب شود.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPoint {
    /// مختصهٔ افقی
    pub x: f64,
    /// مختصهٔ عمودی
    pub y: f64,
    /// برچسب (۰ = نقطهٔ معمولی، ۱ = نقطهٔ مهم)
    pub flags: u32,
}

fn repr_c() {
    let p = CPoint { x: 1.5, y: -2.5, flags: 1 };
    println!("  ساختار: {p:?}");
    println!("  اندازه: {} بایت (16 + 4 + 4 بایت padding)", std::mem::size_of::<CPoint>());
    println!("  تراز (alignment): {}", std::mem::align_of::<CPoint>());

    // تبدیل به بایت‌ها برای فرستادن به C (مثل شبکه یا فایل)
    // SAFETY: CPoint یک نوع «plain old data» است؛ هیچ اشاره‌گر یا نوع پیچیده‌ای ندارد.
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts((&p as *const CPoint).cast::<u8>(), std::mem::size_of::<CPoint>())
    };
    println!("  بایت‌های اول: {:?}", &bytes[..8]);
    println!("  (نمایش تلخ واقعیت: «ساختار» در حافظه فقط بایت است)");

    // 🌟 ترفند آموزشی: ترتیب بایت‌ها به معماری بستگی دارد!
    println!("  ترتیب بایت معماری: {}", if cfg!(target_endian = "little") { "کم‌بایت‌اول (x86/ARM)" } else { "بزرگ‌بایت‌اول" });
    println!("  ⚠️ برای فرستادن روی شبکه، همیشه از to_be_bytes استفاده کن (نه این‌طور)");
    println!("     مثال: {}", 0x1234u16.to_be_bytes().iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "));
}

// ═══════════════════════════════════════════════════════════════════════════
/// خواندن بدون بررسی محدوده — نسخهٔ unsafe.
///
/// این تابع *دقیقاً* همان کاری را می‌کند که `slice[i]` می‌کند، منهای چکِ محدوده.
/// (پس سریع‌تر است، ولی اگر ایندکس غلط بدهی → رفتار تعریف‌نشده.)
///
/// # Safety
///
/// فراخوان **باید** تضمین کند:
/// * `ptr` اشاره‌گر معتبر به حافظهٔ سالم (allocated) باشد،
/// * و `index < تعداد عناصر آن حافظه`.
///
/// بخش `# Safety` تزیینی نیست — **قرارداد رسمی** است. هر کسی که این تابع را
/// صدا بزند، باید در کامنت `SAFETY:` دلیل خودش را بنویسد.
unsafe fn read_unchecked(ptr: *const i32, index: usize) -> i32 {
    // در Rust 2024، حتی داخل تابع `unsafe` هم باید بلوک unsafe بنویسیم
    // (لینت `unsafe_op_in_unsafe_fn` که در lib.rs فعال کردیم).
    // مزیتش: «مستندسازی موضعی» — دقیقاً می‌بینی کدام عمل خطرناک است.
    unsafe { *ptr.add(index) }
}

/// پوشش امن: خودش محدوده را چک می‌کند و بعد unsafe را صدا می‌زند.
fn safe_get(slice: &[i32], index: usize) -> Result<i32, String> {
    if index >= slice.len() {
        return Err(format!("ایندکس {index} خارج از محدودهٔ {} عنصری است", slice.len()));
    }
    // SAFETY: شرط بالا تضمین کرد که index < len است و slice حافظهٔ معتبر دارد.
    Ok(unsafe { read_unchecked(slice.as_ptr(), index) })
}

fn unsafe_fns() {
    let data = [10, 20, 30];
    println!("  safe_get(data, 1) = {:?}", safe_get(&data, 1));
    println!("  safe_get(data, 9) → {:?}", safe_get(&data, 9).unwrap_err());

    // اگر خودت مطمئن باشی، می‌توانی مستقیم صدا بزنی (با مسئولیت خودت):
    // SAFETY: می‌دانیم طول data سه است و ایندکس ۲ داخل محدوده است.
    let v = unsafe { read_unchecked(data.as_ptr(), 2) };
    println!("  فراخوانی unsafe با تضمین خودمان: {v}");

    // 🧠 الگوی معماری: «هستهٔ unsafe + پوستهٔ امن» (safe wrapper)
    //    کاربران کتابخانه هرگز unsafe نمی‌بینند، ولی ما کارایی را از دست نمی‌دهیم.
    println!("  📌 درس: توابع unsafe را خصوصی بگذار و یک API امن رویشان بساز");
}

// ═══════════════════════════════════════════════════════════════════════════
/// بافر مالک با تخصیص دستی — برای دیدن «مبحث مالکیت در سطح بایت».
pub struct ManualBuffer {
    /// اشاره‌گر به حافظهٔ تخصیص‌داده‌شده
    ptr: *mut u8,
    /// طول منطقی
    len: usize,
}

impl ManualBuffer {
    /// ساخت بافر همراستا با تعداد بایت داده‌شده (پر از صفر).
    pub fn zeroed(len: usize) -> Self {
        assert!(len > 0, "طول باید مثبت باشد");
        // تخصیص با هم‌راستایی 8 (تضمین‌شده برای هر نوع معمول)
        let layout = std::alloc::Layout::from_size_align(len, 8).expect("چیدمان نامعتبر");
        // SAFETY: layout معتبر است و اندازه‌اش صفر نیست.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout); // مثل «حافظه تمام شد» در C
        }
        ManualBuffer { ptr, len }
    }

    /// نمایی امن روی محتوا.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr به len بایت تخصیص‌یافته و مقداردهی‌شده اشاره می‌کند.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// نمای تغییرپذیر امن.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: هم‌چنان که بالا؛ و `&mut self` دسترسی انحصاری را تضمین می‌کند.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// طول
    pub fn len(&self) -> usize {
        self.len
    }
}

// 🔑 نکتهٔ کلیدی: `Drop` کار آزادسازی را خودکار می‌کند → کاربر نمی‌تواند
// اشتباهاً فراموش کند (همان چیزی که در C به‌عنوان memory leak می‌شناسیم).
impl Drop for ManualBuffer {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.len, 8).expect("چیدمان نامعتبر");
        // SAFETY: همان layout و همان اشاره‌گری که در ساخت تخصیص شد.
        unsafe { std::alloc::dealloc(self.ptr, layout) };
        println!("    (بافر دستی آزاد شد — {len} بایت)", len = self.len);
    }
}

fn manual_memory() {
    {
        let mut buf = ManualBuffer::zeroed(16);
        buf.as_mut_slice()[0] = b'A';
        buf.as_mut_slice()[15] = b'Z';
        println!("  بافر دستی: {} بایت، اول={}، آخر={}", buf.len(), buf.as_slice()[0] as char, buf.as_slice()[15] as char);
    } // اینجا Drop خودکار آزاد می‌کند
    println!("  بالا: پیام Drop را دیدی؟ — RAII یعنی «مالکیت = مسئولیت آزادسازی»");

    // ── تبدیل مالکیت Box ↔ اشاره‌گر خام ──
    let boxed = Box::new(vec![1, 2, 3]);
    // `into_raw` مالکیت را «رها» می‌کند: نه Drop صدا زده می‌شود نه حافظه آزاد
    let raw: *mut Vec<i32> = Box::into_raw(boxed);
    println!("  Box::into_raw → اشاره‌گر خام {:p}", raw);

    // و برگرداندنش — **باید** برگردانی، وگرنه نشت می‌کند
    // SAFETY: raw از همان Box::into_raw آمده و یک بار بیشتر برگردانده نمی‌شود.
    let back: Box<Vec<i32>> = unsafe { Box::from_raw(raw) };
    println!("  Box::from_raw → {back:?} (مالکیت برگشت؛ Drop دوباره فعال است)");

    println!("  📌 این الگو در FFI لازم است: وقتی C باید مالکیت را نگاه دارد");
}

// ═══════════════════════════════════════════════════════════════════════════
fn checklist() {
    let items = [
        "مقدار اشاره‌گر تهی (null) نیست",
        "حافظه واقعاً تخصیص یافته و مقداردهی شده",
        "نوعِ درآیندشده با دادهٔ واقعی می‌خواند (type punning ممنوع)",
        "دسترسی خارج از محدوده نزنیم (index < len)",
        "هم‌راستایی (alignment) رعایت شده",
        "هیچ دو قرض متضادی روی همان داده نسازیم (aliasing)",
        "برای `&mut`، دسترسی انحصاری از راه دیگری نباشد",
        "اختلاف ترتیب/اندازه بین Rust و C (`repr(C)`) حل شده",
        "حافظه دو بار آزاد نشود (double free)",
        "رشته‌های C نول‌ترمینال باشند",
    ];
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }
    println!("\n  📚 مطالعهٔ بیشتر: «Rustonomicon» — کتاب رسمی دربارهٔ unsafe");
    println!("  🛠️ ابزار: `cargo miri` منطق unsafe تو را به‌صورت رسمی بررسی می‌کند (نایتلی)");
    println!("  ✅ اما ۹۹٪ کدهای Rust *هیچ* unsafeای لازم ندارند — این را فقط برای یادگیری دیدی");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn raw_pointer_read_write() {
        let mut v = [1i32, 2, 3];
        let p = v.as_mut_ptr();
        // SAFETY: ایندکس‌ها داخل محدوده‌اند.
        unsafe {
            *p.add(2) = 30;
        }
        assert_eq!(v, [1, 2, 30]);
    }

    #[test]
    fn safe_wrapper_handles_edge_cases() {
        let data = [1, 2, 3];
        assert_eq!(safe_get(&data, 0), Ok(1));
        assert!(safe_get(&data, 3).is_err());
        assert!(safe_get(&[], 0).is_err());
    }

    #[test]
    fn manual_buffer_reads_and_writes() {
        let mut b = ManualBuffer::zeroed(4);
        b.as_mut_slice()[0] = 9;
        assert_eq!(b.as_slice(), &[9, 0, 0, 0]);
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn box_round_trip_preserves_data() {
        let b = Box::new(String::from("داده"));
        let raw = Box::into_raw(b);
        // SAFETY: اشاره‌گر از into_raw آمده و یک بار برمی‌گردد.
        let back = unsafe { Box::from_raw(raw) };
        assert_eq!(*back, "داده");
    }

    #[test]
    fn repr_c_layout_is_stable() {
        assert_eq!(std::mem::size_of::<CPoint>(), 24);
        assert_eq!(std::mem::align_of::<CPoint>(), 8);
    }
}
