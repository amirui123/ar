//! # 🎨 رنگ در ترمینال
//!
//! ترمینال‌ها «کدهای فرار» (escape codes) ANSI را می‌فهمند. رنگ با کاراکتر
//! فرار `ESC` (معادل `\x1b` یا `\u{1b}`) و بعدش یک کد شروع می‌شود:
//!
//! ```text
//! \x1b[31m  → متن قرمز شروع شود
//! \x1b[0m   → همه‌چیز ریست شود
//! ```
//!
//! ## سه درس مهم این فایل
//! 1. **چرا `const`؟** رشته‌های ثابت در بخش دادهٔ باینری می‌نشینند؛ صفر کپی.
//! 2. **محیط را احترام بگذار:** استاندارد [`NO_COLOR`](https://no-color.org) و
//!    تشخیص «آیا خروجی ترمینال است؟» (`IsTerminal`) تا وقتی خروجی را در فایل
//!    می‌ریزیم فایل پر کد فرارِ خراب نشود.
//! 3. **`AtomicI8`**: یک حالت سراسری بدون `unsafe` و بدون قفل — برای فلگ «آیا رنگ فعال است».

use std::io::IsTerminal;
use std::sync::atomic::{AtomicI8, Ordering};

// ── کدهای پایه ───────────────────────────────────────────────────────────────
/// برگرداندن همه‌چیز به حالت عادی.
pub const RESET: &str = "\x1b[0m";
/// درشت (bold).
pub const BOLD: &str = "\x1b[1m";
/// کم‌رنگ (dim) — برای متن‌های حاشیه‌ای.
pub const DIM: &str = "\x1b[2m";
/// مورب.
pub const ITALIC: &str = "\x1b[3m";
/// زیرخط.
pub const UNDERLINE: &str = "\x1b[4m";
/// قرمز — خطا/فوری.
pub const RED: &str = "\x1b[31m";
/// سبز — موفقیت/انجام‌شده.
pub const GREEN: &str = "\x1b[32m";
/// زرد — هشدار/بالا.
pub const YELLOW: &str = "\x1b[33m";
/// آبی — معمولی.
pub const BLUE: &str = "\x1b[34m";
/// بنفش.
pub const MAGENTA: &str = "\x1b[35m";
/// فیروزه‌ای.
pub const CYAN: &str = "\x1b[36m";
/// خاکستری روشن.
pub const GRAY: &str = "\x1b[90m";

/// حالت اجباری رنگ: `0` = خودکار، `1` = همیشه روشن، `-1` = همیشه خاموش.
///
/// 🔑 **ترفند:** `static` متغیرهای سراسری در Rust به‌طور پیش‌فرض تغییرپذیر
/// نیستند (`static mut` نیاز به `unsafe` دارد). راه درست، استفاده از اتمیک‌هاست:
/// بدون قفل، بدون `unsafe`، و امن بین تردها.
static FORCE: AtomicI8 = AtomicI8::new(0);

/// رنگ را به‌صورت دستی روشن/خاموش کن (مثلاً با سوییچ `--no-color`).
pub fn set_enabled(on: bool) {
    // `Relaxed` کافی است: فقط یک فلگ تنظیماتی است، نه همگام‌سازی داده.
    FORCE.store(if on { 1 } else { -1 }, Ordering::Relaxed);
}

/// آیا باید رنگ تولید کنیم؟
///
/// ترتیب اولویت:
/// 1. تنظیم دستی (`--color` / `--no-color`)
/// 2. متغیر محیطی `NO_COLOR` (استاندارد غیررسمی ولی پرکاربرد)
/// 3. آیا خروجی یک ترمینال واقعی است؟ (اگر در pipe/فایل باشد، رنگ نه)
pub fn enabled() -> bool {
    match FORCE.load(Ordering::Relaxed) {
        1 => true,
        -1 => false,
        _ => std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal(),
    }
}

/// متن را با کد رنگ داده‌شده بپوشان (اگر رنگ فعال باشد).
///
/// خروجی یک `String` است. صفت `impl Into<Cow<str>>` را ندیدیم چون می‌خواهیم
/// سادگی را حفظ کنیم؛ در کد حرفه‌ای می‌توان از [`std::borrow::Cow`] استفاده کرد
/// تا در حالت خاموش، هیچ تخصیص حافظه‌ای رخ ندهد (به `docs/07` نگاه کن).
pub fn paint(text: &str, code: &str) -> String {
    if enabled() {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// میان‌بر: درشت
pub fn bold(text: &str) -> String {
    paint(text, BOLD)
}

/// میان‌بر: سبز
pub fn green(text: &str) -> String {
    paint(text, GREEN)
}

/// میان‌بر: قرمز
pub fn red(text: &str) -> String {
    paint(text, RED)
}

/// میان‌بر: زرد
pub fn yellow(text: &str) -> String {
    paint(text, YELLOW)
}

/// میان‌بر: خاکستری کم‌رنگ
pub fn dim(text: &str) -> String {
    paint(text, DIM)
}

/// میان‌بر: فیروزه‌ای
pub fn cyan(text: &str) -> String {
    paint(text, CYAN)
}

/// میان‌بر: بنفش
pub fn magenta(text: &str) -> String {
    paint(text, MAGENTA)
}

/// حذف همهٔ کدهای فرار ANSI از یک رشته.
///
/// **چرا لازم است؟** برای محاسبهٔ «عرض واقعی» یک سلول جدول. اگر متن رنگی باشد،
/// `"\x1b[31mسلام\x1b[0m".chars().count()` جواب غلط می‌دهد چون کاراکترهای فرار
/// هم شمرده می‌شوند. این تابع با یک «ماشین حالت» کوچک (state machine) آن‌ها را رد می‌کند.
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // ESC دیده شد → منتظر '[' باش
            if chars.peek() == Some(&'[') {
                chars.next(); // '[' را بخور
                // پارامترها و حرف پایانی دستور را رد کن
                for c in chars.by_ref() {
                    // حرف‌های ASCII در بازهٔ @..~ پایان دستور هستند
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            // اگر ESC تنها بود، خودش را هم حذف می‌کنیم (چیز مفیدی نیست)
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        let colored = format!("{RED}سلام{RESET} دنیا");
        assert_eq!(strip_ansi(&colored), "سلام دنیا");
        assert_eq!(strip_ansi("بی‌رنگ"), "بی‌رنگ");
        // دو رنگ پشت سر هم و یک کد ۲۴ بیتی:
        assert_eq!(strip_ansi("\x1b[1m\x1b[38;5;208mقرمز\x1b[0m"), "قرمز");
    }

    #[test]
    fn enabled_respects_manual_switch() {
        set_enabled(false);
        assert!(!enabled());
        assert_eq!(paint("x", RED), "x"); // در حالت خاموش، هیچ کد فراری اضافه نمی‌شود
        set_enabled(true);
        assert_eq!(paint("x", RESET), format!("{RESET}x{RESET}"));
        // برگرداندن به حالت خودکار برای بقیهٔ تست‌ها (ترتیب اجرای تست‌ها تضمینی نیست،
        // ولی این تست عملاً اتمیکِ سراسری را تغییر می‌دهد → در تست‌های هم‌زمان مراقب باش)
        FORCE.store(0, Ordering::Relaxed);
    }
}
