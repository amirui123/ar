//! # دموی عمرها (Lifetimes)
//!
//! **عمر = برچسبی روی مرجع‌ها که می‌گوید «این قرض تا کجا معتبر است».**
//!
//! 💡 خبر خوب: در ۹۹٪ کدها **لازم نیست عمر بنویسی** — کامپایلر خودش استنتاج
//! می‌کند (سه قاعدهٔ «حذف عمر»). این فایل مواردی را نشان می‌دهد که واقعاً
//! باید دست به کار شوی.
//!
//! ## سه قاعدهٔ حذف عمر (elision)
//! 1. هر ورودی مرجع، عمر مستقل خودش را می‌گیرد: `fn f(a: &str, b: &str)` → `<'a,'b>`.
//! 2. اگر فقط **یک** ورودی مرجعی باشد، خروجی همان عمر را می‌گیرد.
//! 3. اگر `&self` یا `&mut self` باشد، خروجی عمر `self` را می‌گیرد.


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::fmt;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) کِی لازم نیست عمر بنویسی"));
    elision();

    println!("\n{}", color::bold("۲) کِی لازم است: دو ورودی مرجع"));
    two_inputs();

    println!("\n{}", color::bold("۳) عمر در ساختار (struct با مرجع)"));
    structs_with_refs();

    println!("\n{}", color::bold("۴) `'static` — چه معنایی دارد و چه ندارد"));
    statics();

    println!("\n{}", color::bold("۵) عمر در impl و متدها"));
    impls();

    println!("\n{}", color::bold("۶) راه‌های فرار: چرا گاهی String بهتر از &str است"));
    escape_hatches();
}

// ═══════════════════════════════════════════════════════════════════════════
/// قاعدهٔ ۲: یک ورودی مرجع → خروجی همان عمر را دارد (بدون نوشتن چیزی!).
fn first_word(s: &str) -> &str {
    // معادل پنهانش: fn first_word<'a>(s: &'a str) -> &'a str
    s.split_whitespace().next().unwrap_or("")
}

/// قاعدهٔ ۳: `&self` → خروجی عمر self را می‌گیرد.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    /// اینجا `&self` است ⇒ خروجی عمر self را دارد ⇒ نیازی به نوشتن `'a` نیست.
    ///
    /// 🪤 دام مهم: `&self.input[self.pos..]` اگر `pos` **وسط یک کاراکتر چندبایتی**
    /// بیفتد، پنیک می‌کند (`byte index N is not a char boundary`).
    /// راه درست: `get()` که `Option` می‌دهد.
    fn rest(&self) -> &str {
        self.input.get(self.pos..).unwrap_or("")
    }
}

fn elision() {
    let text = "سلام دنیای قشنگ";
    let w = first_word(text);
    println!("  first_word(text) = «{w}»");
    println!("  (هیچ عمری در امضا نوشته نشد، ولی درست کار می‌کند)");

    let p = Parser { input: text, pos: 5 };
    println!("  Parser.rest() = «{}»", p.rest());
    println!("  (خروجی به عمر self گره خورد — قاعدهٔ سوم)");

    // 🪤 نمونهٔ همان دام (بدون پنیک، چون چک می‌کنیم):
    let risky = Parser { input: "سلام", pos: 5 }; // بایت ۵ وسط «ا» (بایت‌های ۴..۶) است
    println!("  is_char_boundary(5) = {}", risky.input.is_char_boundary(5));
    println!("  get(5..) = {:?}، ولی unsafe برش مستقیم پنیک می‌کرد", risky.input.get(5..));
    println!("  rest() به‌جای پنیک، امن برگرداند: «{}»", risky.rest());
}

// ═══════════════════════════════════════════════════════════════════════════
/// ❌ این تابع کامپایل نمی‌شود، چون کامپایلر نمی‌داند خروجی از کدام ورودی می‌آید:
///
/// ```compile_fail
/// fn longest(a: &str, b: &str) -> &str {
///     if a.len() > b.len() { a } else { b }
/// }
/// ```
///
/// ✅ راه‌حل: بگو «هر دو یک عمر دارند» → `<'a>` روی هر دو و روی خروجی.
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.chars().count() > b.chars().count() { a } else { b }
}

/// راه‌حل دوم: خروجی را «مستقل» کن و یک مقدار مالک برگردان (اگر کارایی مهم نیست).
fn longest_owned(a: &str, b: &str) -> String {
    if a.chars().count() > b.chars().count() { a.to_string() } else { b.to_string() }
}

fn two_inputs() {
    let a = "کوتاه";
    let b = "این یکی طولانی‌تر است";
    println!("  longest = «{}»", longest(a, b));

    // 🌟 ترفند: عمرها می‌توانند *متفاوت* باشند تا وقتی که کوتاه‌ترین کافی باشد.
    //    اینجا خروجی فقط تا وقتی «x» زنده است معتبر است.
    {
        let x = String::from("سن");
        let y = String::from("y طولانی‌تر");
        let r = longest(&x, &y);
        println!("  در بلوک داخلی: «{r}»");
    } // x و y اینجا می‌میرند و r هم با آن‌ها می‌رود

    println!("  longest_owned = «{}»", longest_owned(a, b));
    println!("  (نسخهٔ owned کمی کندتر ولی هیچ دردسر عمری ندارد)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// یک مفسر (view) روی متن — بدون هیچ کپی.
#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    number: usize,
    text: &'a str,
}

impl fmt::Display for Line<'_> {
    // ⚠️ اینجا `'_` را دیدی؟ یعنی «عمرش برای من مهم نیست» — میان‌بر خوانا
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>3}│ {}", self.number, self.text)
    }
}

impl<'a> Line<'a> {
    /// طول کاراکتری خط — خروجی به عمر ربطی ندارد.
    fn width(&self) -> usize {
        self.text.chars().count()
    }
}

fn structs_with_refs() {
    let document = "خط اول\nخط دوم طولانی‌تر\nخط سوم";
    // متن *قرض* داده می‌شود؛ هیچ کپی‌ای از سند رخ نمی‌دهد
    let lines: Vec<Line<'_>> = document
        .lines()
        .enumerate()
        .map(|(i, text)| Line { number: i + 1, text })
        .collect();

    println!("  سند بدون کپی، فقط «نما»:");
    for l in &lines {
        println!("    {l}   (عرض {})", l.width());
    }

    // 📌 قاعدهٔ مهم: هرچه `Line` بسازیم، عمرش به `document` گره خورده است.
    //    اگر document را جابه‌جا یا حذف کنی، Lineها باطل می‌شوند.
    println!("  ⚠️ این نماها تا وقتی `document` زنده است معتبرند (کامپایلر چک می‌کند)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// یک «پیکربندی» با مقادیر ثابت درون برنامه.
#[derive(Debug)]
struct Config<'a> {
    /// نام برنامه؛ همیشه از رشتهٔ ثابت می‌آید
    app: &'a str,
    /// کاربر ممکن است مقدار پویا بدهد → String لازم است
    user: String,
}

/// فقط وقتی ورودی واقعاً `'static` باشد، کامپایل می‌شود.
fn from_static(name: &'static str) -> Config<'static> {
    Config { app: name, user: "مهمان".into() }
}

fn statics() {
    // `'static` یعنی «این مرجع تا آخر عمر برنامه معتبر است».
    // رشته‌های ثابت متن برنامه `&'static str` هستند.
    let s: &'static str = "من در فایل اجرایی حک شده‌ام";
    println!("  {s}");

    let cfg = from_static("ar");
    println!("  {cfg:?}");

    // ⚠️ اشتباه رایج: `'static` به‌معنای «همیشه برای همیشه» نیست!
    //    `T: 'static` یعنی «هیچ مرجع کوتاه‌عمری داخلش نیست» —
    //    String هم `'static` است (چون هیچ قرضی ندارد).
    let owned: String = "مال من".into();
    let boxed: Box<dyn fmt::Debug + 'static> = Box::new(owned.clone());
    println!("  String هم 'static است (چون هیچ قرضی ندارد): {boxed:?}");

    // ترفند: برای «دور ریختن» عمر یک مرجع محلی و ساختن مقدار طولانی‌عمر:
    let leaked: &'static str = Box::leak(owned.into_boxed_str());
    println!("  Box::leak → {leaked}");
    println!("  ⚠️ نشتِ حافظه: این حافظه تا پایان برنامه آزاد نمی‌شود (آگاهانه!)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// ظرفی که متن را «قرض» می‌گیرد و می‌تواند کمی دستکاری کند.
struct Trimmer<'a> {
    source: &'a str,
}

impl<'a> Trimmer<'a> {
    /// سازنده؛ عمر ورودی و ساختار یکی می‌شود.
    fn new(source: &'a str) -> Self {
        Trimmer { source }
    }

    /// نسخهٔ بریده — باز هم برشی از همان منبع (خروجی عمر `'a` دارد، نه عمر self).
    ///
    /// `&self` عمر *جدا* از `'a` دارد. با نوشتن `-> &'a str` می‌گوییم
    /// «خروجی از خودِ منبع می‌آید، پس تا وقتی منبع زنده است معتبر است».
    fn trimmed(&self) -> &'a str {
        self.source.trim()
    }

    /// نسخهٔ «وابسته به self» — فرض کن بخواهیم حالت داخلی نگه داریم.
    /// خروجی این‌جا نمی‌تواند `'a` باشد، چون از محاسبهٔ درونی می‌آید.
    fn describe(&self) -> String {
        format!("«{}» ({} کاراکتر)", self.trimmed(), self.trimmed().chars().count())
    }
}

fn impls() {
    let raw = "   متن با فاصلهٔ اضافه   ";
    let t = Trimmer::new(raw);

    // برش حاصل، *بعد* از رفتن `t` هم زنده است — چون از `raw` می‌آید!
    let kept: &str = t.trimmed();
    drop(t);
    println!("  بعد از حذف Trimmer، برش هنوز معتبر است: «{kept}»");
    println!("  (چون عمرش به `raw` گره خورده، نه به trımer)");

    // دموی describe
    let t2 = Trimmer::new(raw);
    println!("  {}", t2.describe());
}

// ═══════════════════════════════════════════════════════════════════════════
/// ۳) از `Cow` استفاده کن: یا قرض بده یا مالک شو — بدون تخصیص در مسیر رایج.
///
/// (این تابع را بیرون آوردیم تا تست‌ها هم بتوانند صدایش بزنند.)
fn ensure_prefix<'a>(s: &'a str, prefix: &str) -> std::borrow::Cow<'a, str> {
    if s.starts_with(prefix) {
        std::borrow::Cow::Borrowed(s) // صفر تخصیص
    } else {
        std::borrow::Cow::Owned(format!("{prefix}{s}"))
    }
}

fn escape_hatches() {
    // چهار «راه فرار» از دردسر عمر، به ترتیب اولویت:
    //
    // ۱) مقدار مالک برگردان (ساده‌ترین؛ فقط اگر کپی گران نباشد اشتباه است)
    fn copy_version(s: &str) -> String {
        s.to_uppercase()
    }

    // ۲) شاخص (index) برگردان به‌جای مرجع
    fn find_word_index(haystack: &str, needle: &str) -> Option<usize> {
        haystack.find(needle)
    }

    // ۴) داده را داخل ساختار نگه دار (owned) به‌جای مرجع

    let s = "سلام";
    println!("  copy_version: {}", copy_version(s));
    println!("  find_word_index: {:?}", find_word_index("سلام دنیا", "دنیا"));
    println!("  Cow (بدون تغییر): {:?}", ensure_prefix("ar-tasks", "ar-"));
    println!("  Cow (با تغییر):   {:?}", ensure_prefix("tasks", "ar-"));

    println!("  📌 ترتیب توصیه: ۱) `&str` پارامتر بگیر ۲) String برگردان ۳) شاخص ۴) Cow در مسیرهای داغ");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn longest_returns_one_of_inputs() {
        let a = String::from("aa");
        let b = String::from("bbbb");
        let r = longest(&a, &b);
        assert_eq!(r, "bbbb");
        // و r به‌قدر کوتاه‌ترین ورودی زنده است (کامپایلر تضمین می‌کند)
    }

    #[test]
    fn line_view_borrows_document() {
        let doc = String::from("یک\nدو");
        let lines: Vec<Line<'_>> = doc.lines().enumerate().map(|(i, text)| Line { number: i + 1, text }).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].text, "دو");
        assert_eq!(lines[0].width(), 2);
    }

    #[test]
    fn trimmer_result_outlives_trimmer() {
        let raw = "  hi  ";
        let t = Trimmer::new(raw);
        let kept = t.trimmed();
        drop(t);
        assert_eq!(kept, "hi"); // هنوز معتبر است چون عمرش به raw است
    }

    #[test]
    fn cow_borrows_when_possible() {
        use std::borrow::Cow;
        let c = ensure_prefix("ar-x", "ar-");
        assert!(matches!(c, Cow::Borrowed(_)));
        let c2 = ensure_prefix("y", "ar-");
        assert!(matches!(c2, Cow::Owned(_)));
    }

    #[test]
    fn static_str_is_forever() {
        let s: &'static str = from_static("ar").app;
        assert_eq!(s, "ar");
    }
}
