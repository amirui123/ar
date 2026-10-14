//! # دموی ایتراتورها — قلب سبک کار در Rust
//!
//! ایدهٔ اصلی: ایتراتورها **تنبل** (lazy) هستند. تا وقتی یک «مصرف‌کننده»
//! (collect, fold, sum, count …) نگذاری، هیچ کاری انجام نمی‌شود.
//!
//! ```text
//!    داده                    آداپتورها (تنبل)               مصرف‌کننده
//!   [1,2,3] ──► .map() ──► .filter() ──► .take(2) ────► .collect()
//!                ↑ هیچ کاری تا اینجا انجام نشده ────────► اینجا اجرا می‌شود
//! ```
//!
//! و بزرگ‌ترین مزیت: **بدون حلقه‌های دستی، بدون خطای index خارج از محدوده.**

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) تنبلی — کار فقط وقتی انجام می‌شود که بگویی"));
    laziness();

    println!("\n{}", color::bold("۲) آداپتورهای پرکاربرد"));
    adapters();

    println!("\n{}", color::bold("۳) مصرف‌کننده‌ها"));
    consumers();

    println!("\n{}", color::bold("۴) ایتراتور دلخواه — خودت بساز"));
    custom_iterator();

    println!("\n{}", color::bold("۵) ایتراتور نامتناهی و برداشت از آن"));
    infinite();

    println!("\n{}", color::bold("۶) الگوهای حرفه‌ای: fold، scan، گروه‌بندی، جمع خطاها"));
    pro_patterns();
}

// ═══════════════════════════════════════════════════════════════════════════
fn laziness() {
    // یک شمارنده که هر بار فراخوانی‌شدنش را ثبت می‌کند
    let calls = std::cell::Cell::new(0);

    let iter = [1, 2, 3].iter().map(|x| {
        calls.set(calls.get() + 1);
        x * 2
    });

    println!("  بلافاصله بعد از map: تعداد فراخوانی = {}  ← صفر!", calls.get());

    // اینجا تازه اجرا می‌شود (مصرف‌کننده)
    let sum: i32 = iter.sum();
    println!("  بعد از sum: جمع={sum}، تعداد فراخوانی = {}", calls.get());

    // ⚡ مزیت واقعی: با `take` می‌توانی از یک زنجیرهٔ سنگین، فقط بخش لازم را اجرا کنی
    let heavy_calls = std::cell::Cell::new(0);
    let first_two: Vec<i32> = (0..)
        .map(|x| {
            heavy_calls.set(heavy_calls.get() + 1);
            x * x
        })
        .filter(|x| x % 2 == 0)
        .take(2)
        .collect();
    println!("  از ایتراتور نامتناهی، دو نتیجه: {first_two:?} (فقط {} بار محاسبه)", heavy_calls.get());
}

// ═══════════════════════════════════════════════════════════════════════════
fn adapters() {
    let nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    println!("  map(x*10)          : {:?}", nums.iter().map(|x| x * 10).collect::<Vec<_>>());
    println!("  filter(زوج)        : {:?}", nums.iter().filter(|x| *x % 2 == 0).collect::<Vec<_>>());
    println!("  filter_map         : {:?}", nums.iter().filter_map(|x| (x % 3 == 0).then_some(x * 100)).collect::<Vec<_>>());
    println!("  flat_map           : {:?}", nums.iter().take(3).flat_map(|x| [x, x]).collect::<Vec<_>>());
    println!("  enumerate          : {:?}", ["الف", "ب"].iter().enumerate().collect::<Vec<_>>());
    println!("  zip                : {:?}", [1, 2, 3].iter().zip(["یک", "دو", "سه"]).collect::<Vec<_>>());
    println!("  chain              : {:?}", [1, 2].iter().chain([3, 4].iter()).collect::<Vec<_>>());
    println!("  take/skip          : {:?}", nums.iter().skip(2).take(3).collect::<Vec<_>>());
    println!("  take_while(<5)     : {:?}", nums.iter().take_while(|x| **x < 5).collect::<Vec<_>>());
    println!("  skip_while(<5)     : {:?}", nums.iter().skip_while(|x| **x < 5).collect::<Vec<_>>());
    println!("  step_by(3)         : {:?}", nums.iter().step_by(3).collect::<Vec<_>>());
    println!("  rev()              : {:?}", nums.iter().rev().take(3).collect::<Vec<_>>());
    println!("  peekable (نگاه بدون مصرف):");

    let mut peek = nums.iter().peekable();
    while let Some(x) = peek.next() {
        let next = peek.peek().map(|v| v.to_string()).unwrap_or_else(|| "—".into());
        print!("    {x} (بعدی: {next})");
        if x > &2 { break; }
    }
    println!();

    // `scan` — مثل fold ولی هر گام را هم بیرون می‌دهد (جمع تجمعی)
    let cumulative: Vec<i32> = nums.iter().take(5).scan(0, |acc, x| { *acc += x; Some(*acc) }).collect();
    println!("  scan (جمع تجمعی)   : {cumulative:?}");

    // ترفند: `inspect` برای اشکال‌زدایی وسط زنجیره (کد را تغییر نمی‌دهد)
    let _ = nums
        .iter()
        .inspect(|x| if **x == 3 { println!("    (inspect دید: {x})") })
        .count();
}

// ═══════════════════════════════════════════════════════════════════════════
fn consumers() {
    let nums = [4, 8, 15, 16, 23, 42];

    println!("  sum       = {}", nums.iter().sum::<i32>());
    println!("  product(۳ تا) = {}", nums.iter().take(3).product::<i32>());
    println!("  count     = {}", nums.iter().count());
    println!("  min/max   = {:?}/{:?}", nums.iter().min(), nums.iter().max());
    println!("  max_by_key (باقیماندهٔ تقسیم بر ۱۰) = {:?}", nums.iter().max_by_key(|x| *x % 10));
    println!("  any(>40)  = {}", nums.iter().any(|x| *x > 40));
    println!("  all(زوج؟) = {}", nums.iter().all(|x| x % 2 == 0));
    println!("  find(>15) = {:?}", nums.iter().find(|x| **x > 15));
    println!("  position  = {:?}", nums.iter().position(|x| *x == 16));
    println!("  fold      = {}", nums.iter().fold(0, |acc, x| acc + x));
    println!("  reduce (بدون مقدار اولیه) = {:?}", nums.iter().copied().reduce(|a, b| a.max(b)));
    println!("  nth(2)    = {:?}", nums.iter().nth(2));
    println!("  last      = {:?}", nums.iter().last());

    // `partition` — یک پیمایش، دو خروجی
    let (evens, odds): (Vec<i32>, Vec<i32>) = nums.iter().partition(|x| *x % 2 == 0);
    println!("  partition → زوج={evens:?} فرد={odds:?}");

    // `unzip` — برعکس zip
    let pairs = [(1, 'a'), (2, 'b')];
    let (ns, cs): (Vec<i32>, Vec<char>) = pairs.iter().copied().unzip();
    println!("  unzip → {ns:?} و {cs:?}");

    // `for_each` — وقتی فقط می‌خواهی کاری کنی (بازدهی ندارد)
    nums.iter().take(2).for_each(|x| print!("    for_each: {x}\n"));

    // ترفند: collect به انواع مختلف — بسته به نوعی که اعلام می‌کنی
    use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
    let as_set: HashSet<i32> = nums.iter().copied().collect();
    let as_btree: BTreeSet<i32> = nums.iter().copied().collect();
    let as_map: HashMap<i32, i32> = nums.iter().map(|x| (*x, x * x)).collect();
    let as_deque: VecDeque<i32> = nums.iter().copied().collect();
    let as_result: Result<Vec<i32>, std::num::ParseIntError> = ["1", "2", "3"].iter().map(|s| s.parse()).collect();
    println!("  collect → HashSet {} عنصر، BTreeSet {} عنصر، HashMap {} عنصر، VecDeque {} عنصر",
        as_set.len(), as_btree.len(), as_map.len(), as_deque.len());
    println!("  collect::<Result<Vec<_>>> = {:?}  ← اولین خطا را کوتاه می‌کند!", as_result);

    // ترفند: `collect::<String>()` روی chars
    let s: String = "سلام".chars().rev().collect();
    println!("  برعکس‌کردن رشته با ایتراتور: {s}");
}

// ═══════════════════════════════════════════════════════════════════════════
/// ایتراتور دلخواه: دنبالهٔ فیبوناچی.
///
/// 📌 الگو: یک struct که «وضعیت» را نگه می‌دارد + پیاده‌سازی `Iterator`
/// که هر بار `next()` را جلو می‌برد و `Option` برمی‌گرداند (`None` = پایان).
struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    // «نوع وابسته» (associated type): نوع آیتم‌ها
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.a; // مقدار فعلی
        let new_b = self.a.saturating_add(self.b); // جلوگیری از سرریز
        self.a = self.b;
        self.b = new_b;
        Some(next) // هرگز تمام نمی‌شود (نامتناهی)
    }

    // ⚡ ترفند بهینه‌سازی: اگر اندازهٔ باقی‌مانده را بدانی، `size_hint` را دقیق بده
    // تا `collect` بتواند از قبل حافظه بگیرد.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None) // می‌دانیم نامتناهی است
    }
}

/// ایتراتور شمارنده با اندازهٔ دقیق — نمونهٔ کامل `ExactSizeIterator`.
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        Some(self.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0 as usize;
        (n, Some(n)) // دقیق!
    }
}

impl ExactSizeIterator for Countdown {}
impl std::iter::FusedIterator for Countdown {} // تضمین: بعد از None همیشه None

fn custom_iterator() {
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("  فیبوناچی: {fibs:?}");

    // چون size_hint دقیق دادیم، این‌ها هم کار می‌کنند:
    let fib_sum: u64 = Fibonacci::new().take(8).sum();
    println!("  جمع ۸ تای اول: {fib_sum}");

    let countdown: Vec<u32> = Countdown(4).collect();
    println!("  شمارش معکوس: {countdown:?}");

    // ترفند: پیاده‌سازی `DoubleEndedIterator` یعنی `rev()` هم کار می‌کند.
    // اینجا برای Countdown رایگان است:
    let cd = Countdown(3);
    println!("  len با ExactSizeIterator: {}", cd.len());
    // و اگر نیاز داری، از `range` داخلی استفاده کن (که همهٔ این صفات را دارد):
    let r: Vec<i32> = (1..=5).rev().step_by(2).collect();
    println!("  range با rev+step_by: {r:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn infinite() {
    // ۱) تکرار یک مقدار
    let repeated: Vec<&str> = std::iter::repeat("ها").take(3).collect();
    println!("  repeat: {}", repeated.concat());

    // ۲) تکرار یک بستار — 🔑 `repeat_with` برای تولیدکننده‌های stateful
    let mut counter = 0;
    let powers: Vec<i32> = std::iter::repeat_with(|| { counter += 1; 2i32.pow(counter - 1) }).take(6).collect();
    println!("  repeat_with (توان‌های ۲): {powers:?}");

    // ۳) ایتراتور بی‌پایان با `successors` — زنجیرهٔ حالت
    let collatz: Vec<u64> = std::iter::successors(Some(27u64), |&n| {
        if n == 1 { None } else if n % 2 == 0 { Some(n / 2) } else { Some(3 * n + 1) }
    })
    .take(12)
    .collect();
    println!("  زنجیرهٔ کولاتز از ۲۷: {collatz:?} … (طول کامل: {} گام)", collatz_length(27));

    // ۴) `from_fn` — ایتراتور دلخواه بدون نوشتن struct
    let mut n = 0;
    let squares: Vec<u32> = std::iter::from_fn(|| { n += 1; (n <= 5).then(|| n * n) }).collect();
    println!("  from_fn (مربع‌ها): {squares:?}");

    // ⚠️ دام: ایتراتور نامتناهی بدون `take` → `collect` تا بی‌نهایت می‌رود!
    println!("  (هرگز `.collect()` روی ایتراتور نامتناهی بدون take نزن)");
}

fn collatz_length(start: u64) -> usize {
    let mut n = start;
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

// ═══════════════════════════════════════════════════════════════════════════
fn pro_patterns() {
    // ── الگوی ۱: گروه‌بندی با `entry` + ایتراتور ──
    use std::collections::BTreeMap;
    let words = ["سیب", "موز", "آناناس", "انار", "آلو", "به"];
    let mut by_first: BTreeMap<char, Vec<&str>> = BTreeMap::new();
    for w in words {
        // `chars().next()` → Option<char>؛ با unwrap_or جایگزین امن می‌گذاریم
        by_first.entry(w.chars().next().unwrap_or('؟')).or_default().push(w);
    }
    println!("  گروه‌بندی با حرف اول:");
    for (ch, list) in &by_first {
        println!("    {ch} → {list:?}");
    }

    // ── الگوی ۲: جمع کردن همهٔ خطاها (نه فقط اولی) ──
    let inputs = ["1", "دو", "3", "چهار"];
    // `partition` روی Result: موفق‌ها و خطاها را جدا می‌کند
    let (oks, errs): (Vec<Result<i32, std::num::ParseIntError>>, Vec<Result<i32, std::num::ParseIntError>>) =
        inputs.iter().map(|s| s.parse::<i32>()).partition(std::result::Result::is_ok);
    let good: Vec<i32> = oks.into_iter().filter_map(std::result::Result::ok).collect();
    println!("  پارس دسته‌ای → درست: {good:?}، غلط: {} مورد", errs.len());

    // ── الگوی ۳: `try_fold` — fold با امکان خطا (کوتاه‌کننده) ──
    let total: Result<i32, &str> = [10, 20, 30].iter().try_fold(0i32, |acc, x| {
        acc.checked_add(*x).ok_or("سرریز!")
    });
    println!("  try_fold جمع: {total:?}");

    // ── الگوی ۴: «پنجرهٔ لغزان» بدون تخصیص ──
    let prices = [10.0, 11.5, 9.8, 12.0, 13.5];
    let moving_avg: Vec<f64> = prices
        .windows(3)
        .map(|w| w.iter().sum::<f64>() / w.len() as f64)
        .collect();
    println!("  میانگین متحرک (پنجرهٔ ۳): {moving_avg:?}");

    // ── الگوی ۵: `flat_map` برای «تخت‌کردن» تودرتویی ──
    // ⚠️ قید مهم: در آرایه، همهٔ سطرها باید *هم‌طول* باشند `[[i32; N]; M]`.
    //    اگر طول‌ها فرق کنند، باید `Vec<Vec<i32>>` یا برش بگیریم:
    let matrix: [&[i32]; 3] = [&[1, 2], &[3, 4], &[5]];
    let flat: Vec<i32> = matrix.iter().flat_map(|row| row.iter()).copied().collect();
    println!("  flat_map روی ماتریس: {flat:?}");

    // ── الگوی ۶: شماره‌گذاری با enumerate + join ──
    let numbered = ["الف", "ب", "ج"]
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}.{s}", i + 1))
        .collect::<Vec<_>>()
        .join("  ");
    println!("  enumerate+join: {numbered}");

    // ── الگوی ۷: `chain` برای ترکیب داده از منابع مختلف ──
    let all: Vec<&i32> = [1, 2].iter().chain([3, 4].iter()).chain([5].iter()).collect();
    println!("  chain از سه منبع: {all:?}");
    println!("  (کم‌هزینه‌ترین راه ادغام داده‌های پراکنده — بدون هیچ کپی)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn laziness_holds() {
        let count = std::cell::Cell::new(0);
        let it = (0..10).map(|x| { count.set(count.get() + 1); x });
        assert_eq!(count.get(), 0, "map تنبل است");
        let v: Vec<i32> = it.take(3).collect();
        assert_eq!(v.len(), 3);
        assert_eq!(count.get(), 3, "فقط ۳ بار اجرا شد، نه ۱۰");
    }

    #[test]
    fn custom_iterators_are_consistent() {
        assert_eq!(Fibonacci::new().take(8).collect::<Vec<_>>(), vec![0, 1, 1, 2, 3, 5, 8, 13]);
        assert_eq!(Countdown(3).collect::<Vec<_>>(), vec![2, 1, 0]);
        assert_eq!(Countdown(3).len(), 3);
    }

    #[test]
    fn collect_into_result_short_circuits() {
        let r: Result<Vec<i32>, _> = ["1", "x", "3"].iter().map(|s| s.parse::<i32>()).collect();
        assert!(r.is_err());
    }

    #[test]
    fn collatz_from_27_is_111_steps() {
        assert_eq!(collatz_length(27), 111); // نتیجهٔ معروف
    }
}
