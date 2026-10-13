//! # دموی الگوها (Patterns) — قوی‌ترین ابزار روزمرهٔ Rust
//!
//! «الگو» یک شکل از *داده* است؛ `match` روی الگوها کار می‌کند:
//! تفکیک enum، تجزیهٔ ساختار، شرط‌های نگهبان، محدوده‌ها، `@` برای
//! نگه‌داشتن مقدار در کنار شرط، و … .
//!
//! 💡 نکتهٔ کلیدی: کامپایلر **جامعیت** (exhaustiveness) را چک می‌کند.
//! اگر یک حالت را جا بگذاری، برنامه کامپایل نمی‌شود — این تنها ایمنی «زمان کامپایل» است.

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) match روی enum — جامعیت اجباری"));
    on_enums();

    println!("\n{}", color::bold("۲) تجزیهٔ struct و tuple"));
    destructuring();

    println!("\n{}", color::bold("۳) نگهبان (guard)، بازه‌ها و `@`"));
    guards_ranges();

    println!("\n{}", color::bold("۴) الگوهای برش (slice)"));
    slices();

    println!("\n{}", color::bold("۵) `if let`، `while let`، `let else` و `matches!`"));
    sugar();

    println!("\n{}", color::bold("۶) الگوهای مرجع و جعبه‌ها (ref/box)"));
    refs_and_boxes();
}

// ═══════════════════════════════════════════════════════════════════════════
/// یک درخت حساب — نمونهٔ کلاسیک «enum با داده».
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    /// عدد صحیح
    Num(i64),
    /// جمع
    Add(Box<Expr>, Box<Expr>),
    /// ضرب
    Mul(Box<Expr>, Box<Expr>),
    /// منفی
    Neg(Box<Expr>),
}

impl Expr {
    /// محاسبهٔ مقداری درخت (ارزیابی بازگشتی).
    fn eval(&self) -> i64 {
        match self {
            // الگوی «تفکیک با نام‌گذاری»
            Expr::Num(n) => *n,
            Expr::Add(l, r) => l.eval() + r.eval(),
            Expr::Mul(l, r) => l.eval() * r.eval(),
            Expr::Neg(x) => -x.eval(),
        }
        // نکته: هیچ حالت «default» نداریم؛ وگرنه اگر نوعی به Expr اضافه شود،
        // کامپایلر ما را مطلع نمی‌کرد. الگوهای صریح = ایمنی بیشتر.
    }

    /// نسخهٔ خوانا با پرانتز (تبدیل درخت به متن).
    fn show(&self) -> String {
        match self {
            Expr::Num(n) => n.to_string(),
            Expr::Add(l, r) => format!("({} + {})", l.show(), r.show()),
            Expr::Mul(l, r) => format!("({} × {})", l.show(), r.show()),
            Expr::Neg(x) => format!("-{}", x.show()),
        }
    }

    /// مرور بازگشتی روی همهٔ برگ‌ها.
    fn leaves(&self, out: &mut Vec<i64>) {
        match self {
            Expr::Num(n) => out.push(*n),
            _ => {
                // برای راحتی، هر گره‌ای که برگ نیست:
                if let Expr::Add(l, r) | Expr::Mul(l, r) = self {
                    // 🌟 ترفند: الگوی «یا» (or-pattern) — دو الگو با یک بدنه،
                    // و نیم‌بندها (bindings) باید در هر دو یکسان باشند (از Rust 1.65).
                    l.leaves(out);
                    r.leaves(out);
                } else if let Expr::Neg(x) = self {
                    x.leaves(out);
                }
            }
        }
    }
}

fn on_enums() {
    let e = Expr::Add(
        Box::new(Expr::Num(2)),
        Box::new(Expr::Mul(Box::new(Expr::Num(3)), Box::new(Expr::Neg(Box::new(Expr::Num(4)))))),
    );

    println!("  درخت: {}", e.show());
    println!("  مقدار: {}", e.eval());

    let mut leaves = Vec::new();
    e.leaves(&mut leaves);
    println!("  برگ‌ها: {leaves:?}");
    assert_eq!(e.eval(), 2 + 3 * -4);
}

// ═══════════════════════════════════════════════════════════════════════════
/// یک ساختار نمونه — با فیلدهای عمومی تا از بیرون هم بشود تجزیه کرد.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// مختصهٔ افقی
    pub x: f64,
    /// مختصهٔ عمودی
    pub y: f64,
}

/// درجهٔ کارت
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank {
    /// عدد ۲..۱۰
    Number(u8),
    /// سرباز/بی‌بی/شاه
    Face(char),
    /// آس
    Ace,
}

fn destructuring() {
    let p = Point { x: 3.0, y: 4.0 };

    // تجزیهٔ ساختار — همان چیزی که در تُپل‌ها/دیکشنری‌ها آرزو داشتی!
    let Point { x, y } = p;
    println!("  نقطه: x={x}, y={y} (فاصله={:.2})", (x * x + y * y).sqrt());

    // می‌توانی بعضی فیلدها را نادیده بگیری یا تغییرنام بدهی:
    let Point { x: horizontal, y: _ } = p;
    println!("  با تغییرنام و نادیده‌گرفتن: horizontal={horizontal}");

    // در `match` هم می‌شود مستقیم الگو داد:
    match p {
        Point { x: 0.0, y: 0.0 } => println!("  مبدأ مختصات است"),
        Point { x: 0.0, y } => println!("  روی محور عمودی، y={y}"),
        Point { x, y: 0.0 } => println!("  روی محور افقی، x={x}"),
        Point { x, y } => println!("  نقطهٔ دلخواه ({x}, {y})"),
    }

    // tuple: با `..` می‌شود وسط‌ها را رد کرد
    let tuple = (1, "دو", 3.0, 'چ');
    let (first, .., last) = tuple;
    println!("  از tuple: اول={first} آخر={last}");

    // ترفند: `let` می‌تواند ساختار تودرتو را هم باز کند
    let nested = Some((1, Some(2)));
    if let Some((a, Some(b))) = nested {
        println!("  تودرتو: a={a}, b={b}");
    }

    // و کارت‌ها:
    for r in [Rank::Ace, Rank::Number(10), Rank::Face('K')] {
        let label = match r {
            Rank::Ace => "آس".to_string(),
            Rank::Number(n @ 2..=9) => format!("عدد {n}"),
            Rank::Number(n) => format!("عدد بزرگ {n}"),
            Rank::Face(c) => format!("شخصیت {c}"),
        };
        println!("  کارت: {label}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
fn guards_ranges() {
    for n in [-5, 0, 7, 42, 1000] {
        // ۱) بازه‌ها داخل الگو
        let kind = match n {
            i64::MIN..=-1 => "منفی",
            0 => "صفر",
            1..=9 => "تک‌رقمی",
            10..=99 => "دورقمی",
            _ => "بزرگ",
        };

        // ۲) نگهبان (guard): شرط دلخواهی روی مقدار نگاشته‌شده
        let extra = match n {
            x if x % 2 == 0 => "زوج",
            _ => "فرد",
        };

        // ۳) `@` = هم مقدار را بگیر، هم شرط بگذار
        let repr = match n {
            v @ 1..=10 => format!("کوچک({v})"),
            v @ 40..=50 => format!("وسط({v})"),
            other => format!("بقیه({other})"),
        };

        println!("  n={n:<5} → {kind}، {extra}، {repr}");
    }

    // الگوی «بازگشتی» برای بازیابی خطا:
    let results: [std::result::Result<i32, &str>; 3] = [Ok(1), Err("پرید"), Ok(3)];
    let ok_count = results.iter().filter(|r| matches!(r, Ok(_))).count();
    println!("  تعداد Ok: {ok_count}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn slices() {
    let list = [1, 2, 3, 4, 5];

    for window in [&list[..], &[][..], &list[..1], &list[..3], &list[2..]] {
        let count = window.len();
        let msg = match window {
            [] => "خالی".to_string(),
            [a] => format!("یک عنصر: {a}"),
            [a, b] => format!("دو عنصر: {a},{b}"),
            [first, .., last] => format!("چند عنصر: اول={first} آخر={last} ({count} عنصر)"),
        };
        println!("  {msg}");
    }

    // ترفند رایج: «head/tail» بدون هیچ کپی‌ای
    let mut rest: &[i32] = &list;
    let mut sum = 0;
    while let [head, tail @ ..] = rest {
        sum += *head;
        rest = tail;
    }
    println!("  جمع با الگوی head/tail = {sum}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn sugar() {
    let maybe: Option<i32> = Some(5);

    // if let — فقط یک حالت را می‌خواهیم
    if let Some(v) = maybe {
        println!("  if let → {v}");
    }

    // if let … else
    if let Some(v) = None::<i32> {
        println!("  این چاپ نمی‌شود: {v}");
    } else {
        println!("  if let/else → حالت دیگر");
    }

    // let … else — «یا زودتر برگرد» (پایدار از Rust 1.65)
    let val = get_env_number("AR_DEMO_NUMBER").unwrap_or(12);
    println!("  let-else → مقدار {val}");

    // 🌟 ترفند طلایی: `?` روی Option یا Result = «اگر هیچ بود، از تابع بیرون بیا».
    //    کوتاه‌ترین راه برای نوشتن کد زنجیره‌ای.
    let chained: Option<i32> = maybe.map(|v| v * 2).filter(|v| *v > 0).inspect(|v| println!("  inspect → {v}"));
    println!("  زنجیرهٔ Option → {chained:?}");

    // matches! — ماکروی ساده ولی فوق‌العاده کاربردی
    let c = 'ب';
    println!(
        "  matches! → «{c}» حرف است؟ {}",
        matches!(c, 'آ'..='ی' | 'a'..='z' | 'A'..='Z')
    );

    // الگو در پارامتر تابع! (به‌جای `let`)
    fn sum_point(Point { x, y }: Point) -> f64 {
        x + y
    }
    println!("  الگو در پارامتر تابع → {}", sum_point(Point { x: 1.5, y: 2.5 }));
}

/// نمونهٔ «let-else» واقعی — الگویی که خوانایی را زیاد می‌کند.
fn get_env_number(key: &str) -> Option<i64> {
    let Ok(raw) = std::env::var(key) else {
        // ⬅️ let-else: اگر الگو نخواند، بدنهٔ else *باید* واگرا (diverge) باشد:
        //    return، break، continue یا panic.
        return None;
    };
    let Ok(n) = raw.trim().parse::<i64>() else {
        return None;
    };
    Some(n)
}

// ═══════════════════════════════════════════════════════════════════════════
fn refs_and_boxes() {
    let value = String::from("چیز");

    // الگوی مرجع: `&x` مقدار را *قرض* می‌گیرد (نه کپی)
    match &value {
        // `s` اینجا &String است
        s => println!("  قرض: {s}"),
    }

    // در Rust 2024، قواعد «مرگونت» الگو هوشمندتر شده:
    // وقتی روی یک مرجع الگو می‌زنی، دیگر لازم نیست همه‌جا `&` و `ref` بنویسی.
    let opt = Some(String::from("جعبه‌ای"));
    match opt {
        // در Rust 2024، `Some(s)` اینجا هنوز `s: String` می‌دهد (move)
        Some(s) => println!("  با move: {s}"),
        None => println!("  خالی"),
    }

    // ترفند: `ref` را فقط وقتی لازم داری که می‌خواهی مقدار را نفروشی
    let opt2 = Some(String::from("قرضی"));
    match &opt2 {
        Some(s) => println!("  با قرض (مدرن، بدون ref): {s}"),
        None => {}
    }
    // همان کار با `ref` صریح (سبک قدیمی ولی هنوز معتبر):
    match opt2 {
        Some(ref s) => println!("  با ref صریح: {s}"),
        None => {}
    }
    // صفت بزرگ `ref`/`ref mut` الگوی‌اند، نه نوع.

    println!("  پایان دموی الگوها");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn expr_eval_is_correct() {
        let e = Expr::Add(Box::new(Expr::Num(1)), Box::new(Expr::Neg(Box::new(Expr::Num(2)))));
        assert_eq!(e.eval(), -1);
        assert_eq!(e.show(), "(1 + -2)");
    }

    #[test]
    fn slice_patterns() {
        let l = [1, 2, 3];
        let msg = match &l[..] {
            [a, .., b] => format!("{a}-{b}"),
            _ => "?".to_string(),
        };
        assert_eq!(msg, "1-3");
    }
}
