//! # 🎓 دموها — آزمایشگاه مفاهیم زبان
//!
//! هر ماژول این پوشه یک مفهوم Rust را **اجرا** می‌کند، نه فقط توضیح می‌دهد.
//! با `cargo run -- demo <موضوع>` همه را ببین.
//!
//! دموها عمداً همه در یک باینری‌اند (نه یک کریت جدا برای هرکدام) تا:
//! * استارت‌اپ سریع باشد،
//! * کدها بتوانند به هم ارجاع بدهند،
//! * و تو با یک دستور همه را اجرا کنی.
//!
//! 📚 برای هر موضوع، مستند متناظر در پوشهٔ `docs/` را ببین.

use std::process::ExitCode;

use crate::cli::Args;
use crate::errors::{AppError, Result};
use crate::{color, table};

pub mod closures;
pub mod collections;
pub mod concurrency;
pub mod errors_demo;
pub mod generics;
pub mod iterator;
pub mod lifetimes;
pub mod macros_demo;
pub mod numeric;
pub mod ownership;
pub mod patterns;
pub mod smart_pointers;
pub mod strings;
pub mod traits_adv;
pub mod unsafe_ffi;

/// فهرست موضوع‌های موجود: (نام، توضیح کوتاه)
pub const TOPICS: &[(&str, &str)] = &[
    ("ownership", "مالکیت، قرض‌گیری، کپی/انتقال (مهم‌ترین مفهوم Rust)"),
    ("patterns", "الگوگیری (match)، if let، until، تجزیهٔ ساختار"),
    ("strings", "String در مقابل &str، UTF-8، کاراکتر/بایت، Cow"),
    ("numeric", "اعداد، سرریز، تبدیل نوع، const fn، اعداد اعشاری"),
    ("collections", "Vec، HashMap، BTreeMap، VecDeque، HashSet و API ورودی"),
    ("iterator", "ایتراتورها، بستارها، زنجیره‌های تنبل و ایتراتور دلخواه"),
    ("generics", "ژنریک‌ها، قیدها، impl Trait، انواع وابسته، اپراتورها"),
    ("traits_adv", "صفت‌های پیشرفته: سوپرتریت، صفت مهرشده، دینامیک/استاتیک"),
    ("closures", "بستارها: Fn/FnMut/FnOnce، move، برگرداندن بستار"),
    ("smart_pointers", "Box، Rc، RefCell، Weak، Arc/Mutex و چرخهٔ حافظه"),
    ("concurrency", "تردها، کانال‌ها، استخر ترد، اشتراک وضعیت"),
    ("lifetimes", "عمرها: چرا لازم‌اند و کِی صریح بنویسیم"),
    ("macros", "ماکروهای اعلانی: macro_rules!، تکرار، §crate"),
    ("unsafe_ffi", "unsafe، اشاره‌گر خام، extern \"C\" و قواعد صحت"),
    ("errors", "مدیریت خطا: Result، ?، خطاهای سفارشی، Box<dyn Error>"),
];

/// اجرای یک دمو بر اساس آرگومان `ar demo <موضوع>`.
///
/// موضوع `all` همه را پشت سر هم اجرا می‌کند (برای تماشای یکجا).
pub fn run_demo(args: &Args) -> Result<ExitCode> {
    let topic = args.arg(0);

    match topic {
        None => {
            print_topic_list();
            Ok(ExitCode::SUCCESS)
        }
        Some("all") => {
            for (name, desc) in TOPICS {
                println!("\n{}", table::section(&format!("{name} — {desc}")));
                run_one(name)?;
            }
            println!("\n{}", color::green("✔ همهٔ دموها اجرا شد."));
            Ok(ExitCode::SUCCESS)
        }
        Some(name) => {
            run_one(name)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// اجرای یکی از موضوع‌ها با نام.
///
/// # خطاها
/// اگر نام موضوع ناشناخته باشد، با پیشنهاد نزدیک‌ترین‌ها خطا می‌دهد.
fn run_one(name: &str) -> Result<()> {
    println!("{}", table::section(name));

    match name {
        "ownership" => ownership::run(),
        "patterns" => patterns::run(),
        "strings" => strings::run(),
        "numeric" => numeric::run(),
        "collections" => collections::run(),
        "iterator" => iterator::run(),
        "generics" => generics::run(),
        "traits_adv" => traits_adv::run(),
        "closures" => closures::run(),
        "smart_pointers" => smart_pointers::run(),
        "concurrency" => concurrency::run(),
        "lifetimes" => lifetimes::run(),
        "macros" => macros_demo::run(),
        "unsafe_ffi" => unsafe_ffi::run(),
        "errors" => errors_demo::run(),
        other => {
            // پیشنهاد گزینه‌های نزدیک (مسافت لِوِنشتاین ساده و بدون وابستگی)
            let mut scored: Vec<(usize, &str)> = TOPICS
                .iter()
                .map(|(n, _)| (distance(other, n), *n))
                .collect();
            scored.sort_by_key(|(d, _)| *d);
            let hints: Vec<&str> = scored.iter().take(3).map(|(_, n)| *n).collect();

            return Err(AppError::Usage(format!(
                "موضوع «{other}» را نمی‌شناسم.\nشاید یکی از این‌ها را می‌خواستی: {}",
                hints.join("، ")
            )));
        }
    }

    Ok(())
}

/// فهرست موضوع‌ها را با رنگ چاپ کن.
fn print_topic_list() {
    println!("{}", table::section("دموهای موجود"));
    let mut t = table::Table::new(["موضوع", "توضیح"]);
    for (name, desc) in TOPICS {
        t.push([color::cyan(name), (*desc).to_string()]);
    }
    print!("{}", t.render());
    println!(
        "{}\n  {} ar demo ownership\n  {} ar demo all",
        color::dim("نمونه:"),
        color::dim("→"),
        color::dim("→")
    );
}

/// فاصلهٔ لِوِنشتاین — برای پیشنهاد «منظورت این بود؟».
///
/// این الگوریتم کلاسیک برنامه‌نویسی پویا (DP) است و اینجا با
/// **صف آرایه‌ای** پیاده شده (بدون ماتریس کامل) تا حافظه‌اش `O(min(n,m))` باشد.
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur: Vec<usize> = vec![0; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            // کم‌ترینِ: حذف، درج، جانشینی
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        // جابه‌جایی نقش‌ها (`std::mem::swap` — بدون کپی!)
        std::mem::swap(&mut prev, &mut cur);
    }

    prev[b.len()]
}

// سرور وب در ماژول جدا زندگی می‌کند ولی اینجا بازصادر می‌شود تا
// استفاده از آن در main.rs کوتاه باشد: `ar::demos::serve`.
pub use crate::web::serve;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basics() {
        assert_eq!(distance("ownrship", "ownership"), 1);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("rust", "rust"), 0);
    }

    #[test]
    fn all_topics_have_unique_names() {
        let mut names: Vec<&str> = TOPICS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "نام موضوع‌ها تکراری است");
    }

    #[test]
    fn every_topic_is_dispatchable() {
        // تست مهم: تضمین می‌کند هر موضوعی که در فهرست هست، واقعاً اجرا می‌شود.
        // (اگر اشتباه تایپی داشته باشیم، اینجا لو می‌رود.)
        for (name, _) in TOPICS {
            run_one(name).unwrap_or_else(|e| panic!("موضوع «{name}» اجرا نشد: {e}"));
        }
    }
}
