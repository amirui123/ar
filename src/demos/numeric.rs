//! # دموی اعداد — سرریز، تبدیل نوع، و دام‌های اعشاری
//!
//! سه قاعده‌ای که هر برنامه‌نویس Rust باید بداند:
//! 1. **سرریز در حالت debug پنیک می‌کند، در release می‌پیچد (wrap).**
//!    پس هرگز به رفتار «پیچیدن» تکیه نکن؛ از `wrapping_*` یا `checked_*` استفاده کن.
//! 2. **`as` یک «تبدیل کور» است** — اطلاعات را بی‌صدا دور می‌ریزد. راه درست:
//!    `TryFrom` / `try_into()`.
//! 3. **اعشاری `f64` دقیق نیست:** `0.1 + 0.2 != 0.3`. مقایسه فقط با اپسیلون.

use std::num::{IntErrorKind, NonZeroU32, Saturating, Wrapping};

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) انواع عددی و محدوده‌ها"));
    types_and_ranges();

    println!("\n{}", color::bold("۲) سرریز — چهار راه برخورد"));
    overflow_matters();

    println!("\n{}", color::bold("۳) تبدیل نوع: `as` = کور، `try_into` = امن"));
    casts();

    println!("\n{}", color::bold("۴) اعداد اعشاری و دام‌هایشان"));
    floats();

    println!("\n{}", color::bold("۵) عملیات بیتی و انواع صفرنشدنی"));
    bits_and_nonzero();

    println!("\n{}", color::bold("۶) const fn — محاسبه در زمان کامپایل"));
    consts();
}

fn types_and_ranges() {
    // اندازه‌ها از std::mem می‌آید (در زمان کامپایل حل می‌شود)
    println!("  i8  : {} بایت، محدودهٔ {}..={}", std::mem::size_of::<i8>(), i8::MIN, i8::MAX);
    println!("  u8  : {} بایت، محدودهٔ {}..={}", std::mem::size_of::<u8>(), u8::MIN, u8::MAX);
    println!("  i32 : {} بایت، محدودهٔ {}..={}", std::mem::size_of::<i32>(), i32::MIN, i32::MAX);
    println!("  i64 : {} بایت (معمولاً هم اندازهٔ اشاره‌گر نیست)", std::mem::size_of::<i64>());
    println!("  usize: {} بایت — اندازهٔ «بر اساس معماری» برای ایندکس‌ها", std::mem::size_of::<usize>());
    println!("  isize: {} بایت", std::mem::size_of::<isize>());
    println!("  f32: {} بایت، f64: {} بایت", std::mem::size_of::<f32>(), std::mem::size_of::<f64>());

    // ترفند: پسوند پیش‌فرض نوع در استنتاج
    let x = 5; // i32 پیش‌فرض است
    let y = 5u8; // با پسوند نوع
    let z = 5_000_000i64; // جداکنندهٔ خواناگر (`_`) — کاملاً برای چشم انسان است
    println!("  x={x} (i32 پیش‌فرض)، y={y} (u8)، z={z} (i64)");
    // ترفند: پیشوندهای مبنای عدد
    println!("  دودویی 0b1010 = {}", 0b1010);
    println!("  هشت‌هشتی 0o17 = {}", 0o17);
    println!("  شانزده‌شانزدهی 0xFF_FF = {}", 0xFF_FF);

    // `as` اینجا: چرا لازم شد؟ چون i64 و i32 دو نوع متفاوت‌اند.
    println!("  i64::MAX را به i64 نمی‌شود به u64 داد بدون تبدیل: {}", i64::MAX as u64);
}

fn overflow_matters() {
    let max = u8::MAX; // 255
    println!("  u8::MAX = {max}");

    // ۱) رفتار پیش‌فرض: در debug پنیک، در release پیچیدن.
    //    پس ما *هرگز* به آن تکیه نمی‌کنیم و صریح می‌نویسیم:
    println!("  wrapping_add(+1)   = {}", max.wrapping_add(1)); // 0
    println!("  saturating_add(+1) = {}", max.saturating_add(1)); // 255 (سیر شده)
    println!("  checked_add(+1)    = {:?}", max.checked_add(1)); // None

    // ۲) overflowing: هم نتیجه، هم اینکه سرریز شد یا نه
    let (value, did_overflow) = max.overflowing_add(1);
    println!("  overflowing_add    = ({value}, سرریز={did_overflow})");

    // ۳) `Wrapping<T>` — وقتی می‌خواهی هش/رمزنگاری با پیچیدن کار کند
    let w = Wrapping(255u8);
    println!("  Wrapping: (255 + 1) % 256 = {}", (w + Wrapping(1)).0);

    // ۴) `Saturating<T>` — وقتی می‌خواهی هیچ‌وقت از محدوده بیرون نزنی (گرافیک/سیگنال)
    let s = Saturating(250u8) + Saturating(100u8);
    println!("  Saturating: {} (به‌جای پنیک یا پیچیدن)", s.0);

    // ترفند: برای «جمع کردن روزمره» در اجرا (نه منطق حساس) `+=` عادی کافی است،
    // ولی در واکشی از ورودی کاربر، همیشه `checked_add` بزن.
    let user_input: i32 = 2_000_000_000;
    match user_input.checked_add(100_000_000) {
        Some(v) => println!("  جمع امن: {v}"),
        None => println!("  جمع امن: سرریز می‌شد! → اینجا خطا می‌دهیم"),
    }

    // ترفند: همیشه `checked_*` برای ضرب در فرمول‌های مالی
    println!("  ضرب امن: {:?}", 1_000_000i64.checked_mul(1_000_000));
}

fn casts() {
    // ❌ خطر: `as` بی‌صدا برش می‌دهد
    let big: i64 = 300;
    let as_u8 = big as u8; // 300 % 256 = 44 — اطلاعات گم شد!
    println!("  300i64 as u8 = {as_u8}  ← ⚠️ برش خاموش");
    println!("  -1i32 as u32 = {}  ← ⚠️ علامت گم شد", -1i32 as u32);

    // ✅ راه امن ۱: try_into (خطا می‌دهد اگر جا نشود)
    let ok: std::result::Result<u8, _> = 200i64.try_into();
    let bad: std::result::Result<u8, _> = 300i64.try_into();
    println!("  200i64.try_into::<u8>() = {ok:?}");
    println!("  300i64.try_into::<u8>() = {:?}", bad.map_err(|e| e.to_string()));

    // ✅ راه امن ۲: با دست خودت چک کن (وقتی پیام خطای بهتری می‌خواهی)
    let n: i64 = -5;
    if let Ok(v) = u32::try_from(n) {
        println!("  u32::try_from(-5) = {v}");
    } else {
        println!("  u32::try_from(-5) = خطا (منفی است!) — پیام خودمان را می‌دهیم");
    }

    // تبدیل اعشاری:
    println!("  3.99f64 as i32 = {}  ← به صفر میل می‌کند، گرد نمی‌کند!", 3.99f64 as i32);
    println!("  3.99f64.round() as i32 = {}  ← گرد شده", 3.99f64.round() as i32);

    // ترفند مهم: `as` در تبدیل *گسترش‌دهنده* (مثل u8→u64) همیشه امن است،
    // ولی برای هر تبدیل *تنگ‌شونده* از try_into استفاده کن.
    let safe = u8::MAX as u64;
    println!("  u8::MAX as u64 = {safe} (همیشه امن)");
}

fn floats() {
    println!("  ❗ 0.1 + 0.2 = {:.20}", 0.1_f64 + 0.2_f64);
    println!("  0.1 + 0.2 == 0.3 ?  {}", (0.1_f64 + 0.2_f64) == 0.3);
    // راه درست برای مقایسه:
    let (a, b) = (0.1_f64 + 0.2_f64, 0.3_f64);
    println!("  با اپسیلون: {}", (a - b).abs() < 1e-10);
    println!("  یا f64::EPSILON*: {}", (a - b).abs() < f64::EPSILON * 10.0);

    // NaN عجیب‌ترین مقدار دنیاست: با هیچ‌چیز برابر نیست، حتی با خودش!
    let nan = f64::NAN;
    println!("  NAN == NAN → {}   (پس با is_nan چک کن)", nan == nan);
    println!("  NAN.is_nan() → {}", nan.is_nan());
    println!("  NAN < 1.0 → {}   (هر مقایسه‌ای با NaN غلط است)", nan < 1.0);

    // توابع کاربردی
    println!("  floor/ceil/round/trunc: {}/{}/{}/{}",
        (-1.5_f64).floor(), (-1.5_f64).ceil(), (-1.5_f64).round(), (-1.5_f64).trunc());
    println!("  fract/abs/signum: {}/{}/{}", 2.75_f64.fract(), (-3.0_f64).abs(), (-3.0_f64).signum());
    println!("  powi/powf/sqrt: {}/{:.3}/{:.3}", 2.0_f64.powi(10), 2.0_f64.powf(0.5), 16.0_f64.sqrt());
    println!("  درجه/رادیان: {:.3}", 180.0_f64.to_radians().to_degrees());
    println!("  min/max/clamp: {}/{}/{}", 1.0_f64.min(2.0), 1.0_f64.max(2.0), 5.0_f64.clamp(0.0, 3.0));

    // ترفند: برای پول از اعشاری استفاده نکن! از عدد صحیح (سنت) استفاده کن.
    let price_cents: i64 = 1999; // برعکس: 19.99 دلار
    println!("  پول با عدد صحیح: {} سنت = {}.{:02} واحد", price_cents, price_cents / 100, price_cents % 100);

    // ترفند: اعداد اعشاری در خروجی با `to_string` ممکن است کوتاه شوند؛
    // `{:.2}` برای نمایش کنترل‌شده بهتر است.
    println!("  نمایش: {} vs {:.2}", 1.0_f64 / 3.0, 1.0_f64 / 3.0);
}

fn bits_and_nonzero() {
    let flags: u8 = 0b0000_1010;
    println!("  flags = {flags:#010b}");
    println!("  تست بیت ۱: {}", flags & 0b0000_0010 != 0);
    println!("  تست بیت ۳: {}", flags & 0b0000_1000 != 0);
    println!("  روشن‌کردن بیت ۰: {:#010b}", flags | 0b0000_0001);
    println!("  خاموش‌کردن بیت ۱: {:#010b}", flags & !0b0000_0010);
    println!("  معکوس‌کردن بیت ۰: {:#010b}", flags ^ 0b0000_0001); // XOR = toggle
    println!("  count_ones: {}، leading_zeros: {}، trailing_zeros: {}",
        flags.count_ones(), flags.leading_zeros(), flags.trailing_zeros());
    println!("  rotate_left(1): {:#010b}، swap_bytes: {:#06x}", flags.rotate_left(1), 0x1234u16.swap_bytes());

    // 🎯 نوع «صفرنشدنی» — ترفندی برای حذف کردن حالت غیرممکن از تایپ
    // در سیستم: `Option<NonZeroU32>` هم اندازهٔ `u32` است (تغییر نیچ = 0)
    println!("  Option<u32> = {} بایت", std::mem::size_of::<Option<u32>>());
    println!("  Option<NonZeroU32> = {} بایت  ← نیچ بهینه‌سازی شده!",
        std::mem::size_of::<Option<NonZeroU32>>());
    let nz = NonZeroU32::new(7).unwrap();
    println!("  NonZeroU32 = {nz}، و new(0) = {:?}", NonZeroU32::new(0));

    // خطای پارس عدد، دقیقاً می‌گوید چه شد:
    let err = "999".parse::<u8>().unwrap_err();
    println!("  خطای پارس: {:?} (kind = {:?})", err, err.kind());
    assert_eq!(*err.kind(), IntErrorKind::PosOverflow);
}

fn consts() {
    // 🌟 `const fn` = تابعی که در زمان *کامپایل* اجرا می‌شود.
    //    نتیجه در فایل اجرایی «حک» می‌شود؛ هزینهٔ صفر در زمان اجرا.
    const fn fib(n: u32) -> u64 {
        // توجه: در const fn حلقه مجاز است (از Rust 1.46)
        let (mut a, mut b) = (0u64, 1u64);
        let mut i = 0;
        while i < n {
            let t = a + b;
            a = b;
            b = t;
            i += 1;
        }
        a
    }

    // این‌ها در زمان کامپایل حساب می‌شوند:
    const F10: u64 = fib(10);
    const F30: u64 = fib(30);
    const TABLE: [u64; 6] = [fib(0), fib(1), fib(2), fib(3), fib(4), fib(5)];

    println!("  const fib(10) = {F10}، fib(30) = {F30}");
    println!("  جدول ثابت: {TABLE:?}");

    // ترفند حرفه‌ای: «assert زمان کامپایل» — اگر شرط برقرار نباشد،
    // خودِ کامپایل شکست می‌خورد (نه تست!). از Rust 1.79 پایدار شد.
    const _: () = assert!(F10 == 55, "فیبوناچی اشتباه محاسبه شد!");

    // ⚡ ترفند: `const { ... }` بلوک ثابت درون‌خطی

    let n = const { 2 + 3 }; // همان جا محاسبه و جای‌گذاری می‌شود
    println!("  const block: {n}");

    // ترفند: `static` در مقابل `const`
    // - const: هر مصرف، یک کپی جدید در جای مصرف (بی‌آدرس)
    // - static: یک جای مشخص در حافظه، آدرس‌دار، عمر 'static
    static GREETING: &str = "سلام از static";
    println!("  {GREETING}");
    println!("  آدرس static ثابت است، آدرس const نه — تفاوت کلیدی‌شان این است");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn overflow_helpers_are_consistent() {
        assert_eq!(u8::MAX.wrapping_add(1), 0);
        assert_eq!(u8::MAX.saturating_add(1), u8::MAX);
        assert_eq!(u8::MAX.checked_add(1), None);
        assert_eq!(u8::MAX.overflowing_add(1), (0, true));
    }

    #[test]
    fn casts_truncate_and_try_into_does_not() {
        assert_eq!(300i64 as u8, 44);
        assert!(u8::try_from(300i64).is_err());
        assert_eq!(u8::try_from(200i64).unwrap(), 200);
    }

    #[test]
    fn nan_is_never_equal() {
        let n = f64::NAN;
        assert!(n != n && n.is_nan());
        assert!(!(n < 0.0) && !(n > 0.0));
    }

    #[test]
    fn const_fn_runs_at_compile_time() {
        const fn sq(x: u32) -> u32 {
            x * x
        }
        const SIXTEEN: u32 = sq(4);
        assert_eq!(SIXTEEN, 16);
    }
}
