//! # دموی رشته‌ها — `String` و `&str` و دنیای UTF-8
//!
//! 🔥 **مهم‌ترین نکته‌ای که تازه‌کارها را زمین می‌زند:**
//! در Rust، رشته **بایت** است، نه آرایهٔ کاراکتر. پس:
//!
//! ```text
//! let s = "سلام";
//! s.len()          → 8   (بایت)
//! s.chars().count()→ 4   (کاراکتر)
//! &s[0..1]         → ❌ PANIC! (وسط کاراکتر بریدی)
//! ```
//!
//! دو نوع رشته داریم:
//! * `&str` → **برش** رشتهٔ ثابت؛ نمی‌توانی بزرگش کنی. ارزان (اشاره‌گر + طول).
//! * `String` → رشتهٔ **مالکِ** قابل رشد روی heap.
//!
//! قاعدهٔ تجربی: **در پارامترهای تابع `&str` بگیر، در ساختارها `String` نگه دار.**

use std::borrow::Cow;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) تفاوت &str و String"));
    two_types();

    println!("\n{}", color::bold("۲) بایت، کاراکتر، گرافم"));
    unicode_trap();

    println!("\n{}", color::bold("۳) برش امن و روش‌های جست‌وجو"));
    safe_slicing();

    println!("\n{}", color::bold("۴) ساخت، رشد و ظرفیت"));
    building();

    println!("\n{}", color::bold("۵) Cow — «شاید صاحبش باشم، شاید قرض بگیرم»"));
    cow_demo();

    println!("\n{}", color::bold("۶) پارس و تبدیل"));
    parsing();
}

fn two_types() {
    // &'static str — اشاره‌گر به داده‌ای که *داخل خود فایل اجرایی* است (پس همیشه زنده)
    let literal: &'static str = "متن ثابت";

    // String — روی heap، مالک، قابل رشد
    let owned: String = literal.to_string(); // یک کپی می‌سازد
    let from: String = String::from(literal); // همان، سبک دیگر

    println!("  ثابت: {literal}");
    println!("  مالک: {owned} | {from}");

    // ترفند: تابعی که &str می‌گیرد، هم String و هم &str را قبول می‌کند
    // (با deref coercion): &String خودکار به &str تبدیل می‌شود.
    takes_str(&owned);
    takes_str(literal);

    // و اگر String داری و لازم نیست کپی کنی، وام بده:
    let borrowed: &str = &owned;
    println!("  قرض از مالک: {borrowed}");
}

fn takes_str(s: &str) {
    println!("    → تابع گرفت: «{s}» (طول {} کاراکتر)", s.chars().count());
}

// ═══════════════════════════════════════════════════════════════════════════
fn unicode_trap() {
    let s = "سلام 😀 Rust";

    println!("  متن: «{s}»");
    println!("  len() (بایت)            = {}", s.len());
    println!("  chars().count()         = {}", s.chars().count());
    println!("  bytes().count()         = {}", s.bytes().count());
    // گرافم = «آنچه کاربر به‌عنوان یک حرف می‌بیند» (الگوریتم خوشه‌بندی یونیکد).
    // std آن را ندارد؛ برای کار جدی crateی مثل `unicode-segmentation` لازم است.
    println!("  (گرافم‌ها را std نمی‌شمارد — نیاز به crate تخصصی)");

    // چاپ کد هر کاراکتر:
    let codes: Vec<String> = s.chars().take(6).map(|c| format!("{c}=U+{:04X}", c as u32)).collect();
    println!("  کدها: {}", codes.join("  "));

    // ترفند: `char` در Rust یک «اسکالر ۳۲ بیتی» است، نه یک بایت.
    println!("  اندازهٔ char در حافظه: {} بایت", std::mem::size_of::<char>());
    println!("  اندازهٔ &str (بورو + طول) = {} بایت", std::mem::size_of::<&str>());
    println!("  اندازهٔ String = {} بایت (اشاره‌گر+طول+ظرفیت)", std::mem::size_of::<String>());
}

// ═══════════════════════════════════════════════════════════════════════════
fn safe_slicing() {
    let s = "برنامه‌نویسی";

    // برش بایتی *فقط* روی مرزهای کاراکتر امن است:
    if s.is_char_boundary(4) {
        println!("  برش امن [0..4] = «{}»", &s[0..4]);
    }

    // ✅ روش امن و توصیه‌شده: با char_indices یا get
    println!("  با get:    {:?}", s.get(0..4));
    println!("  وسط کاراکتر: {:?}  ← None، بدون پنیک!", s.get(0..2));

    // برش بر اساس کاراکتر (کمی کندتر، ولی همیشه امن)
    let first3: String = s.chars().take(3).collect();
    println!("  سه کاراکتر اول: «{first3}»");

    // جست‌وجو
    println!("  contains('نویسی') = {}", s.contains("نویسی"));
    println!("  starts_with('برنامه') = {}", s.starts_with("برنامه"));
    println!("  find('س') = {:?} (آفست بایتی)", s.find('س'));
    println!("  split_whitespace: {:?}", "  یک  دو   سه ".split_whitespace().collect::<Vec<_>>());
    println!("  rsplit().take(1): {:?}", "a/b/c".rsplit('/').take(1).collect::<Vec<_>>());

    // tرفند: trim_* فقط فاصله‌های ASCII را می‌گیرد، نه نیم‌فاصله‌های فارسی!
    println!("  trim: «{}»", "  متن  ".trim());
}

// ═══════════════════════════════════════════════════════════════════════════
fn building() {
    // ساخت با ظرفیت مشخص = پرهیز از تخصیص‌های مکرر (بهینه‌سازی ساده ولی مؤثر)
    let mut out = String::with_capacity(64);

    out.push('ا');                 // یک char
    out.push_str("سلام");          // یک &str
    out.push_str(" دنیا");

    println!("  ساخته‌شده: «{out}» | len={} ظرفیت={}", out.len(), out.capacity());

    // `reserve` برای رشد آماده می‌کند؛ `shrink_to_fit` حافظه اضافه را پس می‌دهد
    out.reserve(128);
    println!("  بعد از reserve(128): ظرفیت={}", out.capacity());
    out.shrink_to_fit();
    println!("  بعد از shrink_to_fit: ظرفیت={}", out.capacity());

    // چسباندن با جداکننده
    let parts = ["یک", "دو", "سه"];
    println!("  join: {}", parts.join(" | "));

    // format! — همان خانوادهٔ println! ولی خروجی String می‌دهد
    let name = "علی";
    let age = 30;
    println!("  format: {}", format!("{name} ({age} ساله)"));
    // تراز و پرکردن (خیلی کاربردی در جدول‌سازی)
    println!("  پرکردن: [{:>8}] [{:<8}] [{:^8}] [{:0>5}]", "راست", "چپ", "وسط", 42);
    // ⚠️ نکتهٔ جالب: std::fmt **جداکنندهٔ هزارگان ندارد** (برخلاف پایتون و JS).
    //    باید خودت بنویسی — که خودش یک تمرین خوب است:
    fn group_digits(n: i64) -> String {
        // ⚠️ تلهٔ کلاسیک: `n.abs()` روی `i64::MIN` **پنیک می‌کند**
        //    (چون +i64::MIN در i64 جا نمی‌شود!). راه درست: `unsigned_abs` → u64.
        let digits = n.unsigned_abs().to_string();
        let mut out = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                out.push(','); // ترفند: هر سه رقم از آخر یک ویرگول
            }
            out.push(c);
        }
        if n < 0 { format!("-{out}") } else { out }
    }
    println!("  عدد با جداکننده: {}  و دقت اعشاری: {:.3}", group_digits(1234567), std::f64::consts::PI);
    println!("  (بزرگ‌ترین عدد قابل نمایش با جداکننده: {})", group_digits(i64::MIN));

    // نمایش دیباگ vs نمایش کاربر
    let v = vec![1, 2, 3];
    println!("  Display ناموجود برای Vec → باید Debug بزنی: {v:?}");
    println!("  Pretty Debug: {v:#?}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn cow_demo() {
    // Cow = Copy-on-Write: «یا قرض می‌دهم یا اگر لازم شد مالک می‌شوم»
    // مزیت: وقتی تغییری لازم نیست، هیچ تخصیص حافظه‌ای رخ نمی‌دهد.
    fn normalize(input: &str) -> Cow<'_, str> {
        if input.contains("  ") {
            // فقط در این حالت کپی می‌سازیم
            Cow::Owned(input.split_whitespace().collect::<Vec<_>>().join(" "))
        } else {
            // در غیر این صورت فقط قرض می‌دهیم — صفر تخصیص!
            Cow::Borrowed(input)
        }
    }

    for s in ["بدون تغییر", "دو  فاصله"] {
        match normalize(s) {
            Cow::Borrowed(b) => println!("  قرض (بدون کپی): «{b}»"),
            Cow::Owned(o) => println!("  مالک (کپی‌شده): «{o}»"),
        }
    }
    println!("  نکته: Cow در امضای تابع = بهترین کارایی بدون پیچیدگی برای کاربر");
}

// ═══════════════════════════════════════════════════════════════════════════
fn parsing() {
    // FromStr — چیزی که `"12".parse::<i32>()` را ممکن می‌کند
    let n: i32 = "42".parse().unwrap();
    println!("  پارس عدد: {n}");

    // ترفند: با `parse()` و turbofish نوع را در محل مشخص می‌کنی
    let pi: f64 = "3.14".parse().unwrap();
    println!("  پارس اعشاری: {pi}");

    // برعکسش: to_string() / format!
    println!("  به رشته: {}", 255.to_string());

    // از بایت‌ها به String (ورودی شبکه/فایل)
    let bytes = vec![0xD8, 0xB3, 0xDB, 0x8C]; // «سی» در UTF-8 → نامعتبر برای مثال
    match String::from_utf8(bytes) {
        Ok(s) => println!("  UTF-8 معتبر: {s}"),
        Err(e) => println!("  بایت نامعتبر (دقیقاً همان چیزی که باید بشود): {}", e.utf8_error()),
    }

    // نسخهٔ بخشنده: از دست دادن بایت‌های نامعتبر
    let lossy = String::from_utf8_lossy(&[0xD8, 0xB3, 0xFF, 0xFE]);
    println!("  from_utf8_lossy: {lossy:?}");

    // حروف بزرگ/کوچک — حواست باشد: برای یونیکد کامل است، ولی «آ» و «ا» یک چیز نیستند
    println!("  uppercase: {}", "Rust".to_uppercase());
    println!("  lowercase: {}", "RUST".to_lowercase());

    // eq_ignore_ascii_case — برای مقایسهٔ سریع و بی‌دردسر ASCII
    println!("  مقایسهٔ بی‌توجه به بزرگی: {}", "HELLO".eq_ignore_ascii_case("hello"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn char_vs_byte_is_the_classic_trap() {
        let s = "سلام";
        assert_eq!(s.len(), 8);
        assert_eq!(s.chars().count(), 4);
        assert!(s.get(0..1).is_none()); // برش وسط کاراکتر → None (نه پنیک!)
        assert_eq!(s.get(0..2), Some("س"));
    }

    #[test]
    fn cow_avoids_allocation_when_possible() {
        fn f(s: &str) -> Cow<'_, str> {
            if s.is_empty() { Cow::Borrowed(s) } else { Cow::Owned(s.to_uppercase()) }
        }
        assert!(matches!(f(""), Cow::Borrowed(_)));
        assert!(matches!(f("a"), Cow::Owned(_)));
    }

    #[test]
    fn utf8_validity() {
        assert!(String::from_utf8(vec![0xF0, 0x9F, 0x98, 0x80]).is_ok()); // 😀
        assert!(String::from_utf8(vec![0xFF]).is_err());
    }
}
