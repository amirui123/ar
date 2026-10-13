//! # دموی بستارها (Closures)
//!
//! بستار = «تابع + محیطی که در آن ساخته شده». سه صفت (trait) برای فراخوانی:
//!
//! | صفت | می‌تواند | مثال |
//! |------|----------|------|
//! | `Fn` | چند بار صدا بزنی، محیط را فقط بخواند | `\|x\| x + step` |
//! | `FnMut` | چند بار صدا بزنی، محیط را تغییر دهد | شمارنده |
//! | `FnOnce` | یک بار (مقدار را مصرف می‌کند) | `\|v\| drop(v)` |
//!
//! 📌 قاعده: هر `Fn` یک `FnMut` است، و هر `FnMut` یک `FnOnce`. پس وقتی
//! پارامتر می‌گیری، **بی‌سخت‌ترین** را بگذار که کافی است (اولویت: Fn → FnMut → FnOnce).


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::cell::RefCell;
use std::collections::HashMap;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) بستار در برابر تابع — تفاوت در «محیط»"));
    basics();

    println!("\n{}", color::bold("۲) سه نوع گرفتن محیط: قرض، قرض تغییرپذیر، انتقال"));
    captures();

    println!("\n{}", color::bold("۳) برگرداندن بستار از تابع"));
    returning();

    println!("\n{}", color::bold("۴) بستار در ساختار و در مجموعه"));
    in_structs();

    println!("\n{}", color::bold("۵) کاربرد واقعی: مرتب‌سازی سفارشی و حافظه‌سازی"));
    real_uses();

    println!("\n{}", color::bold("۶) بستار و تردها (move لازم است)"));
    threads();
}

// ═══════════════════════════════════════════════════════════════════════════
fn basics() {
    // تابع معمولی: هیچ محیطی ندارد
    fn double(x: i32) -> i32 {
        x * 2
    }

    // بستار: می‌تواند از متغیرهای بیرون استفاده کند
    let factor = 10;
    let times = |x: i32| x * factor; // `factor` را از محیط «می‌گیرد»

    println!("  تابع: double(5) = {}", double(5));
    println!("  بستار: times(5) = {} (factor={factor})", times(5));

    // شیوه‌های نوشتن بستار (همه معتبر):
    let a = |x: i32| -> i32 { x + 1 }; // صریح کامل
    let b = |x| x + 1; // استنتاج کامل (اینجا نوع از کاربرد بعدی می‌آید)
    let c = || 42; // بدون پارامتر
    println!("  اشکال نوشتن: {} {} {}", a(1), b(2), c());

    // ⚠️ دام: بستار نوع پارامترش را فقط *یک بار* استنتاج می‌کند.
    // `let id = |x| x; id(1); id("s");` ← خطای کامپایل (دو نوع مختلف).
    println!("  (هر بستار فقط با یک نوع پارامتر کار می‌کند — برخلاف تابع ژنریک)");

    // صفت‌های Fn در عمل:
    let adder = |x: i32| x + 1; // Fn (چند بار قابل استفاده)
    println!("  Fn چندباره: {} {}", adder(1), adder(2));
}

// ═══════════════════════════════════════════════════════════════════════════
fn captures() {
    // ۱) قرض ثابت (Fn)
    let name = String::from("رست");
    let greet = || println!("    سلام {name}!"); // فقظ می‌خواند
    greet();
    greet(); // چند بار مجاز

    // ۲) قرض تغییرپذیر (FnMut)
    let mut count = 0;
    let mut bump = || {
        count += 1;
        count
    };
    println!("    شمارنده: {} {} {}", bump(), bump(), bump());

    // ۳) انتقال مالکیت (FnOnce یا Fn با move)
    let owned = String::from("داده");
    let consume = move || owned.len(); // `move` = مالکیت را به بستار می‌دهد
    println!("    با move: طول = {}", consume());
    // println!("{owned}");  // ← خطای کامپایل: مالکیت منتقل شده است

    // ۴) FnOnce در عمل: بستاری که مقدار را مصرف می‌کند
    let v = vec![1, 2, 3];
    let take_it = move || v; // v را برمی‌گرداند (مصرف می‌کند)
    let got = take_it();
    println!("    FnOnce: {got:?} (دیگر نمی‌توان take_it را صدا زد)");

    // ۵) نکتهٔ ریز: `move` کپی `Copy` هم می‌کند
    let n = 7; // i32: Copy
    let show = move || n; // نسخهٔ کپی در بستار می‌نشیند
    let _n = 100; // متغیر بیرونی را تغییر دادیم…
    println!("    move روی Copy: {} (نه ۱۰۰!)", show());

    // ترفند: `_` در بستار یعنی «این آرگومان را نادیده بگیر»
    let ignore = |_: i32| ();
    ignore(5);
    println!("    بستار با آرگومان نادیده‌گرفته‌شده هم کار کرد");
}

// ═══════════════════════════════════════════════════════════════════════════
/// کارخانهٔ بستار: یک تابع می‌سازد که `n` را اضافه می‌کند.
///
/// 🌟 کلید ماجرا: `move` + `impl Fn` در خروجی. بدون `move`، بستار به متغیر
/// محلی تابع *اشاره* می‌کرد و در لحظهٔ بازگشت، آن متغیر می‌مرد → خطای عمر.
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

/// نسخهٔ پویا: وقتی انواع مختلف بستار را در یک `Vec` می‌خواهی.
fn make_adder_boxed(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + n)
}

fn returning() {
    let add5 = make_adder(5);
    let add10 = make_adder(10);
    println!("  add5(1)={}  add10(1)={}", add5(1), add10(1));
    println!("  (هر کدام محیط مستقل خودش را دارد)");

    // هم‌نوعی لازم نیست چون Box است
    let ops: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),
        make_adder_boxed(100),
        Box::new(|x| x * x),
    ];
    let results: Vec<i32> = ops.iter().map(|f| f(3)).collect();
    println!("  سه بستار متفاوت در یک Vec: {results:?}");

    // ترفند: بستار با حالتی که خودش تغییر می‌کند → `impl FnMut`
    fn counter() -> impl FnMut() -> usize {
        let mut n = 0;
        move || {
            n += 1;
            n
        }
    }
    let mut c = counter();
    println!("  شمارندهٔ برگشتی: {} {} {}", c(), c(), c());
}

// ═══════════════════════════════════════════════════════════════════════════
/// ابزار پردازش متن با «استراتژی» قابل تنظیم.
struct Pipeline {
    steps: Vec<Box<dyn Fn(String) -> String>>,
}

impl Pipeline {
    fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// افزودن یک گام — همین‌جا `dyn Fn` ارزشش را نشان می‌دهد:
    /// می‌توانی هر بستاری، با هر محیطی، در یک `Vec` جمع کنی.
    fn add<F: Fn(String) -> String + 'static>(mut self, f: F) -> Self {
        self.steps.push(Box::new(f));
        self
    }

    fn run(&self, input: &str) -> String {
        // `fold` با رشتهٔ شروع — هر گام روی خروجی گام قبل اجرا می‌شود
        self.steps.iter().fold(input.to_string(), |acc, step| step(acc))
    }
}

fn in_structs() {
    let prefix = "🔸 ".to_string();

    let pipe = Pipeline::new()
        .add(|s: String| s.trim().to_string())
        .add({
            let prefix = prefix.clone(); // بستار بعدی مالک نسخهٔ کپی خودش است
            move |s| format!("{prefix}{s}")
        })
        .add(|s| s.to_uppercase());

    println!("  ورودی خام: «   سلام دنیا   »");
    println!("  خروجی پایپ‌لاین: «{}»", pipe.run("   سلام دنیا   "));

    // 🧠 نمونهٔ کلاسیک «callback»: ثبت رفتار در ساختار
    struct Button {
        on_click: Box<dyn Fn(i32)>,
    }
    let btn = Button {
        on_click: Box::new(|clicks| {
            // اینجا فقط چاپ می‌کنیم؛ در برنامهٔ واقعی به رویداد وصل می‌شود
            println!("    دکمه کلیک شد (شمارش {clicks})");
        }),
    };
    (btn.on_click)(1);
    (btn.on_click)(2);
}

// ═══════════════════════════════════════════════════════════════════════════
fn real_uses() {
    // ۱) مرتب‌سازی با معیار دلخواه — پرکاربردترین بستارِ دنیای واقعی
    let mut people = vec![("علی", 30), ("زهرا", 25), ("رضا", 35), ("مریم", 28)];
    people.sort_by(|a, b| a.1.cmp(&b.1)); // بر اساس سن
    println!("  مرتب بر اساس سن: {people:?}");

    people.sort_by(|a, b| a.0.cmp(b.0)); // بر اساس نام
    // ⚠️ نکته: `sort_by_key(|p| &p.0)` کار *نمی‌کند* (کلید از عنصر قرض می‌گیرد)
    //    — همین یکی از تله‌های معروف Rust است؛ با `sort_by` حلش می‌کنیم.
    println!("  مرتب بر اساس نام: {people:?}");

    // ۲) حافظه‌سازی (memoization) — بستاری که نتیجه‌ها را کش می‌کند
    let cache: RefCell<HashMap<u64, u64>> = RefCell::new(HashMap::new());
    let fib = |n: u64| -> u64 {
        // این بستار `Fn` است (RefCell اجازهٔ تغییر داخلی می‌دهد بدون &mut)
        if let Some(&v) = cache.borrow().get(&n) {
            return v;
        }
        let v = match n {
            0 => 0,
            1 => 1,
            _ => {
                // محاسبهٔ بازگشتی ساده (اینجا صرفاً برای نمایش)
                let (mut a, mut b) = (0u64, 1u64);
                for _ in 0..n {
                    let t = a + b;
                    a = b;
                    b = t;
                }
                a
            }
        };
        cache.borrow_mut().insert(n, v);
        v
    };

    println!("  fib(50) = {}", fib(50));
    println!("  fib(50) بار دوم (از کش) = {}", fib(50));
    println!("  اندازهٔ کش: {} ورودی", cache.borrow().len());

    // ۳) بستار به‌عنوان «شرط» پاس داده‌شده
    fn count_if<T, F: Fn(&T) -> bool>(items: &[T], pred: F) -> usize {
        items.iter().filter(|i| pred(i)).count()
    }
    let nums = [1, 2, 3, 4, 5, 6];
    println!(
        "  زوج‌ها: {}، بزرگ‌تر از ۳: {}",
        count_if(&nums, |n| n % 2 == 0),
        count_if(&nums, |n| *n > 3)
    );

    // ۴) بستاری که با هر فراخوانی تغییر می‌کند (استخر گردشی)
    let pool = ["سرور۱", "سرور۲", "سرور۳"];
    let mut next = 0;
    let mut pick = || {
        let s = pool[next % pool.len()];
        next += 1;
        s
    };
    let picks: Vec<&str> = (0..5).map(|_| pick()).collect();
    println!("  گردشی بین سرورها: {picks:?}");

    // ۵) `retain` با شرط بستاری
    let mut words = vec!["الف".to_string(), "".to_string(), "ب".to_string()];
    words.retain(|w| !w.is_empty());
    println!("  retain بدون خالی‌ها: {words:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn threads() {
    let data = vec![1, 2, 3, 4];

    // ⚠️ بدون `move` این کد کامپایل نمی‌شود: ترد ممکن است بعد از پایان
    // دامنهٔ `data` زنده باشد، پس باید مالکیتش را ببرد.
    let handle = std::thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        sum * 2
    });

    // `join` رشتهٔ اصلی را منتظر می‌گذارد و نتیجه را برمی‌گرداند.
    let result = handle.join().expect("ترد پنیک کرد");
    println!("  نتیجهٔ ترد: {result}");

    // 🎯 ترفند مدرن: «تردهای محدوده‌دار» (`std::thread::scope`، پایدار از ۱.۶۳)
    // نیازی به move و Arc ندارد — تردها می‌توانند دادهٔ محلی را قرض بگیرند!
    let local = vec![10, 20, 30];
    let total = std::thread::scope(|scope| {
        let h1 = scope.spawn(|| local.iter().sum::<i32>());
        let h2 = scope.spawn(|| local.len());
        h1.join().unwrap() + h2.join().unwrap() as i32
    });
    println!("  thread::scope → {total} (بدون move و بدون Arc!)");
    println!("  و `local` هنوز قابل استفاده است: {local:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn adder_captures_environment() {
        let add5 = make_adder(5);
        assert_eq!(add5(1), 6);
        assert_eq!(add5(-5), 0);
        let add10 = make_adder(10);
        assert_eq!(add10(1), 11); // محیط مستقل
    }

    #[test]
    fn fn_mut_counter_increments() {
        fn counter() -> impl FnMut() -> usize {
            let mut n = 0;
            move || {
                n += 1;
                n
            }
        }
        let mut c = counter();
        assert_eq!(c(), 1);
        assert_eq!(c(), 2);
    }

    #[test]
    fn pipeline_composition() {
        let p = Pipeline::new().add(|s: String| s.trim().to_string()).add(|s| s.to_uppercase());
        assert_eq!(p.run("  hi  "), "HI");
    }

    #[test]
    fn memoization_uses_refcell() {
        let cache: RefCell<HashMap<u64, u64>> = RefCell::new(HashMap::new());
        let f = |n: u64| {
            if let Some(v) = cache.borrow().get(&n).copied() {
                return v;
            }
            let v = n * 2;
            cache.borrow_mut().insert(n, v);
            v
        };
        assert_eq!(f(4), 8);
        assert_eq!(f(4), 8);
        assert_eq!(cache.borrow().len(), 1);
    }

    #[test]
    fn scoped_threads_borrow_local_data() {
        let v = vec![1, 2, 3];
        let sum = std::thread::scope(|s| s.spawn(|| v.iter().sum::<i32>()).join().unwrap());
        assert_eq!(sum, 6);
        assert_eq!(v.len(), 3); // v هنوز مالک ماست
    }
}
