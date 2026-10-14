//! # دموی صفت‌های پیشرفته (Traits)
//!
//! صفت = قرارداد. این فایل نشان می‌دهد چطور صفت‌ها به *معماری* برنامه شکل می‌دهند:
//! * متد پیش‌فرض و بازنویسی
//! * صفت والد (supertrait)
//! * پیاده‌سازی «همه‌جاگیر» (blanket impl)
//! * صفت مهرشده (sealed) برای کنترل پیاده‌سازها
//! * ارسال استاتیک (`impl Trait`/ژنریک) در برابر پویا (`dyn Trait`)


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::fmt;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) متدهای پیش‌فرض و بازنویسی"));
    default_methods();

    println!("\n{}", color::bold("۲) صفت والد (supertrait) و ترکیب"));
    supertraits();

    println!("\n{}", color::bold("۳) پیاده‌سازی همه‌جاگیر (blanket impl)"));
    blanket();

    println!("\n{}", color::bold("۴) صفت مهرشده — کنترل اینکه چه کسی پیاده کند"));
    sealed();

    println!("\n{}", color::bold("۵) ارسال استاتیک در برابر پویا (impl/dyn)"));
    dispatch();

    println!("\n{}", color::bold("۶) فرار از «قاعدهٔ یتیم» با newtype"));
    orphan_rule();
}

// ═══════════════════════════════════════════════════════════════════════════
/// صفت «شکل» با متد پیش‌فرض.
trait Shape {
    /// نام شکل (پیاده‌سازها باید بدهند)
    fn name(&self) -> &str;

    /// مساحت (پیاده‌سازها باید بدهند)
    fn area(&self) -> f64;

    /// توضیح — **متد پیش‌فرض**: اگر پیاده‌ساز نخواهد، این استفاده می‌شود.
    fn describe(&self) -> String {
        shape_describe(self)
    }

    /// متد پیش‌فرضی که از متدهای دیگر استفاده می‌کند (گرهٔ قرارداد).
    fn is_larger_than(&self, other: &dyn Shape) -> bool {
        self.area() > other.area()
    }
}

/// پیاده‌سازی «مرجع» توصیف — جدا نوشتیم تا پیاده‌سازها بتوانند
/// نسخهٔ پیش‌فرض را صدا بزنند بدون آنکه در بازگشت بی‌پایان بیفتند.
/// (`S: ?Sized` یعنی هم `&dyn Shape` و هم `&Rect` را می‌پذیرد — ترفند ارگونومی.)
fn shape_describe<S: Shape + ?Sized>(shape: &S) -> String {
    format!("{} با مساحت {:.2}", shape.name(), shape.area())
}

/// دایره.
struct Circle {
    /// شعاع
    r: f64,
}

impl Shape for Circle {
    fn name(&self) -> &str {
        "دایره"
    }
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.r * self.r
    }
    // `describe` را بازنویسی نکردیم → از نسخهٔ پیش‌فرض استفاده می‌کند
}

/// مستطیل — این یکی `describe` را بازنویسی می‌کند.
struct Rect {
    /// عرض
    w: f64,
    /// ارتفاع
    h: f64,
}

impl Shape for Rect {
    fn name(&self) -> &str {
        "مستطیل"
    }
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn describe(&self) -> String {
        // 🧩 «فراخوانی نسخهٔ پیش‌فرض» برای باقی متن.
        //    (اگر `Shape::describe(self)` می‌نوشتیم، بازگشت بی‌پایان می‌شد!)
        format!("[{}×{}] {}", self.w, self.h, shape_describe(self))
    }
}

fn default_methods() {
    let c = Circle { r: 2.0 };
    let r = Rect { w: 3.0, h: 4.0 };

    println!("  {} → {}", c.name(), c.describe());
    println!("  {} → {}", r.name(), r.describe());
    println!("  مستطیل از دایره بزرگ‌تر است؟ {}", r.is_larger_than(&c));
    println!("  (متد پیش‌فرض یک بار نوشته شد، همهٔ شکل‌ها رایگان گرفتنش)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// 🌟 **صفت والد**: هر پیاده‌سازِ `Named` **باید** `Display` هم پیاده کند.
///
/// چرا مهم است؟ چون حالا می‌توانیم بدون هیچ قید اضافه‌ای `{}` بزنیم
/// و بدانیم کار می‌کند.
trait Named: fmt::Display {
    /// نام کوتاه
    fn short_name(&self) -> String;

    /// توضیح کامل — `Self` را به‌عنوان Display استفاده می‌کند (تضمین‌شده!)
    fn banner(&self) -> String {
        format!("║ {self} ({}) ║", self.short_name())
    }
}

/// سرویس پایگاه داده (نمونه).
struct Database {
    /// رشتهٔ اتصال
    url: String,
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "پایگاه‌داده {}", self.url)
    }
}

impl Named for Database {
    fn short_name(&self) -> String {
        "db".to_string()
    }
}

fn supertraits() {
    let db = Database { url: "postgres://localhost/app".into() };
    println!("  {}", db.banner());

    // ترفند: صفت والد + قید ژنریک = تابعی که هم Display دارد هم short_name
    fn log<T: Named>(x: &T) {
        println!("  [لاگ] {} :: {}", x.short_name(), x);
    }
    log(&db);
}

// ═══════════════════════════════════════════════════════════════════════════
/// صفت الحاقی (extension trait) — ساختن متد روی هر نوعی.
trait VecExt {
    /// آیا عنصر مشخصی را دارد؟
    fn has(&self, needle: &str) -> bool;
}

/// 🎯 پیاده‌سازی «همه‌جاگیر»: برای **هر** `Vec<String>` (و هر چیزی که به آن
/// درآیند شود) این متد را اضافه می‌کند — بدون دست‌زدن به std!
impl VecExt for Vec<String> {
    fn has(&self, needle: &str) -> bool {
        self.iter().any(|s| s.eq_ignore_ascii_case(needle))
    }
}

/// یک پیاده‌سازی همه‌جاگیر روی همهٔ انواع قابل‌نمایش — الگوی پرکاربرد در کریت‌ها.
trait ToJsonish {
    /// تبدیل به «متن JSON-مانند» (نمونه)
    fn to_jsonish(&self) -> String;
}

impl<T: fmt::Display> ToJsonish for T {
    fn to_jsonish(&self) -> String {
        format!("\"{}\"", self)
    }
}

fn blanket() {
    let tags = vec!["Rust".to_string(), "CLI".to_string()];
    println!("  VecExt.has(\"rust\") = {}", tags.has("rust"));
    println!("  VecExt.has(\"go\") = {}", tags.has("go"));

    // چون `ToJsonish` برای همهٔ Display-ها پیاده شده، هرچیزی کار می‌کند:
    println!("  to_jsonish: {} {} {}", 42.to_jsonish(), 3.5.to_jsonish(), "سلام".to_jsonish());

    // ⚠️ دام همه‌جاگیری: اگر یک پیاده‌سازی عمومی بنویسی، نمی‌توانی بعداً
    // برای نوع خاصی رفتار متفاوت بدهی (تعارض پیاده‌سازی) — مگر با
    // designهایی مثل «تفکیک صفت کمکی» یا specialization (نایتلی).
    println!("  (پیاده‌سازی همه‌جاگیر = قدرتمند ولی برگشت‌ناپذیر — با احتیاط)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// الگوی «صفت مهرشده»: کاربر بیرونی *نمی‌تواند* پیاده‌سازی کند.
///
/// راز ماجرا: `Sealed` صفتی در یک ماژول خصوصی است که فقط خودمان پیاده‌اش کردیم.
mod private {
    /// صفت مهر — بیرون از این ماژول قابل پیاده‌سازی نیست.
    pub trait Sealed {}

    /// نوع نمونه‌ای که مهر دارد.
    pub struct Internal;
    impl Sealed for Internal {}

    /// و برای عدد صحیح هم.
    impl Sealed for i32 {}
}

/// صفت عمومی که «مهر» دارد.
///
/// چرا لازم است؟ چون با مهر، می‌توانی *بعداً* متد اضافه کنی یا رفتار را تغییر
/// دهی بدون آنکه کد کاربران بشکند. یک ابزار آینده‌نگری (future-proofing).
trait Sealed: private::Sealed {
    /// تبدیل به متن
    fn encode(&self) -> String;
}

impl Sealed for private::Internal {
    fn encode(&self) -> String {
        "internal".into()
    }
}

impl Sealed for i32 {
    fn encode(&self) -> String {
        format!("i32:{self}")
    }
}

fn sealed() {
    println!("  Internal.encode() = {}", private::Internal.encode());
    println!("  42.encode()       = {}", 42i32.encode());
    println!("  ❌ بیرون از این کریت نمی‌تواند Sealed را پیاده کند → قرارداد قفل است");
    println!("     (این دقیقاً الگویی است که std و کتابخانه‌های جدی استفاده می‌کنند)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// صفت «حیوان» — برای مقایسهٔ دو سبک ارسال.
trait Animal {
    /// صدا
    fn sound(&self) -> &'static str;
    /// نام
    fn name(&self) -> &'static str;
}

/// گربه
struct Cat;
impl Animal for Cat {
    fn sound(&self) -> &'static str {
        "میو"
    }
    fn name(&self) -> &'static str {
        "گربه"
    }
}

/// سگ
struct Dog;
impl Animal for Dog {
    fn sound(&self) -> &'static str {
        "واق"
    }
    fn name(&self) -> &'static str {
        "سگ"
    }
}

/// ارسال **استاتیک**: کامپایلر برای هر نوع یک نسخه می‌سازد (تک‌ریخت‌سازی).
/// صفر هزینه در اجرا.
fn static_dispatch<T: Animal>(animal: &T) -> String {
    format!("{} می‌گوید {}", animal.name(), animal.sound())
}

/// ارسال **پویا**: در زمان اجرا از جدول (vtable) پیدا می‌شود.
/// یک نسخهٔ باینری، ولی هر فراخوانی هزینهٔ جست‌وجو دارد.
fn dynamic_dispatch(animal: &dyn Animal) -> String {
    format!("{} می‌گوید {}", animal.name(), animal.sound())
}

fn dispatch() {
    let cat = Cat;
    let dog = Dog;

    println!("  استاتیک: {}", static_dispatch(&cat));
    println!("  استاتیک: {}", static_dispatch(&dog));
    println!("  پویا:    {}", dynamic_dispatch(&cat));
    println!("  پویا:    {}", dynamic_dispatch(&dog));

    // مزیت اصلی `dyn`: می‌توانی انواع *مختلف* را در یک مجموعه بگذاری.
    // (با `impl Animal` نمی‌شود، چون Vec باید یک نوع ثابت داشته باشد.)
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Cat), Box::new(Dog)];
    let sounds: Vec<String> = zoo.iter().map(|a| dynamic_dispatch(a.as_ref())).collect();
    println!("  باغ‌وحش یکدست با dyn: {sounds:?}");

    // ترفند: اندازه‌شان را در حافظه مقایسه کن
    println!(
        "  size_of::<&Cat>()={} و size_of::<&dyn Animal>()={} (اشاره‌گر چاق!)",
        std::mem::size_of::<&Cat>(),
        std::mem::size_of::<&dyn Animal>()
    );
    // اشاره‌گر به dyn دو کلمه است: اشاره‌گر داده + اشاره‌گر vtable.

    // ✅ راهنمای انتخاب:
    //   - در API کتابخانه: ژنریک/impl Trait (صفر هزینه، همه می‌توانند استفاده کنند)
    //   - در مجموعه‌های ناهمگون یا برای کم‌کردن حجم باینری: dyn
    //   - در باینری‌هایی که فایلشان مهم است: dyn (باینری کوچک‌تر)
    println!("  قاعدهٔ سرانگشتی: کتابخانه → استاتیک، مجموعهٔ ناهمگون → پویا");
}

// ═══════════════════════════════════════════════════════════════════════════
fn orphan_rule() {
    // ❌ نمی‌توانی `impl fmt::Display for Vec<f64>` بنویسی:
    //    قاعدهٔ یتیم می‌گوید یا نوع باید مال ما باشد یا صفت.
    //    (هم Display و هم Vec متعلق به std هستند.)

    /// ✅ راه‌حل: newtype (پوشش) — نوع مال ماست، پس اجازه داریم.
    struct Meters(f64);

    impl fmt::Display for Meters {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:.1} متر", self.0)
        }
    }

    println!("  {}", Meters(12.34));
    println!("  ❌ `impl Display for Vec<f64>` غیرممکن است (قاعدهٔ یتیم)");
    println!("  ✅ راه‌حل: newtype — حتی «صفر هزینه در اجرا» چون #[repr(transparent)]");

    #[repr(transparent)]
    struct Wrapper(Vec<f64>);
    impl fmt::Display for Wrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let sum: f64 = self.0.iter().sum();
            write!(f, "مجموع {} عدد = {:.2}", self.0.len(), sum)
        }
    }
    println!("  {}", Wrapper(vec![1.0, 2.5, 3.5]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn default_method_and_override() {
        let c = Circle { r: 1.0 };
        assert!(c.describe().contains("دایره"));
        let r = Rect { w: 2.0, h: 2.0 };
        assert!(r.describe().starts_with("[2×2]"));
        assert!(r.is_larger_than(&c));
    }

    #[test]
    fn extension_trait_works() {
        assert!(vec!["Rust".to_string()].has("rust"));
        assert!(!vec!["Rust".to_string()].has("go"));
    }

    #[test]
    fn blanket_impl_covers_all_display() {
        assert_eq!(42.to_jsonish(), "\"42\"");
        assert_eq!("x".to_jsonish(), "\"x\"");
    }

    #[test]
    fn dispatch_forms_agree() {
        let c = Cat;
        assert_eq!(static_dispatch(&c), dynamic_dispatch(&c));
    }

    #[test]
    fn expr_sealed_impls_exist() {
        assert_eq!(private::Internal.encode(), "internal");
        assert_eq!(7i32.encode(), "i32:7");
    }
}
