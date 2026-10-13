//! # دموی ژنریک‌ها و صفت‌ها — «کد بنویس، برای همه‌چیز»
//!
//! ژنریک یعنی تابع/ساختار را یک بار بنویس و برای انواع مختلف استفاده کن — با
//! **هزینهٔ صفر در زمان اجرا**. دلیلش «تک‌ریخت‌سازی» (monomorphization) است:
//! کامپایلر برای هر نوعِ استفاده‌شده یک نسخهٔ تخصصی می‌سازد. یعنی کد ژنریک
//! *همان* سرعت کد دستی را دارد.


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::fmt;
use std::ops::{Add, AddAssign, Deref, Index, Mul};

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) تابع ژنریک و قیدها (bounds)"));
    generic_functions();

    println!("\n{}", color::bold("۲) ساختار و enum ژنریک"));
    generic_types();

    println!("\n{}", color::bold("۳) صفت با نوع وابسته"));
    associated_types();

    println!("\n{}", color::bold("۴) سربارگذاری اپراتورها"));
    operator_overloading();

    println!("\n{}", color::bold("۵) تبدیل‌ها: From، Into، TryFrom"));
    conversions();

    println!("\n{}", color::bold("۶) impl Trait و const generics"));
    modern_generics();
}

// ═══════════════════════════════════════════════════════════════════════════
/// بزرگ‌ترین عنصر یک برش.
///
/// قیدها (`T: PartialOrd + Copy`) شرط‌هایی هستند که کامپایلر *بررسی می‌کند*:
/// بدون `PartialOrd` نمی‌توانی مقایسه کنی، بدون `Copy` نمی‌توانی مقادیر را کپی کنی.
///
/// ترفند: اگر نمی‌خواهی `Copy` را الزام کنی، با مرجع کار کن:
fn largest_borrowed<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut best = list.first()?; // `?` روی Option در تابعی که Option برمی‌گرداند
    for item in list.iter().skip(1) {
        if item > best {
            best = item;
        }
    }
    Some(best)
}

/// نسخهٔ «مصرف‌کننده» که مقدار را برمی‌گرداند (نیازمند `Copy`).
fn largest_copy<T: PartialOrd + Copy>(list: &[T]) -> Option<T> {
    largest_borrowed(list).copied() // `.copied()` کار Copy→مقدار را راحت می‌کند
}

/// نمونهٔ `where` — وقتی قیدها زیاد می‌شوند، این شکل خیلی خواناتر است.
fn describe_max<T>(list: &[T]) -> String
where
    T: PartialOrd + fmt::Display + Clone,
{
    match largest_borrowed(list) {
        Some(v) => format!("بزرگ‌ترین: {v} (از {} عنصر)", list.len()),
        None => "فهرست خالی است".to_string(),
    }
}

fn generic_functions() {
    println!("  {}", largest_copy(&[3, 7, 2, 9, 4]).map(|v| v.to_string()).unwrap_or_default());
    println!("  {}", largest_copy(&["الف", "ز", "ب"]).unwrap_or("—"));
    println!("  {}", describe_max(&[1.5, 9.25, 3.0]));
    println!("  {}", describe_max(&["کوتاه", "بلندترین"]));

    // ترفند: می‌توانی نوع را در فراخوانی صریح کنی، هر جا ابهام باشد (turbofish)
    let n = largest_copy::<i32>(&[1, 2, 3]);
    println!("  turbofish: {n:?}");

    // ترفند: `impl Trait` در پارامتر — راحت‌ترین راه برای ژنریک ساده
    fn shout(s: impl fmt::Display) -> String {
        format!("{s}!!!")
    }
    println!("  impl Trait در پارامتر: {} و {}", shout(42), shout("سلام"));
}

// ═══════════════════════════════════════════════════════════════════════════
/// پشتهٔ ژنریک — ساختار کلاسیک آموزش数据结构.
#[derive(Debug, Default)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    /// پشتهٔ خالی
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    /// افزودن
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// برداشتن از بالا
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// نگاه به بالا بدون برداشتن
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// تعداد
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// خالی؟
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// `impl` قیددار: این متدها فقط وقتی در دسترس‌اند که T قابل نمایش باشد.
// 🌟 این «تخصص‌یابی مشروط» است — همان کاری که std با `String::from` می‌کند.
impl<T: fmt::Display> Stack<T> {
    /// چاپ پشته از بالا به پایین
    pub fn print_top_down(&self) {
        let rendered: Vec<String> = self.items.iter().rev().map(|i| i.to_string()).collect();
        println!("    [{}]", rendered.join(" ← "));
    }
}

/// یک enum ژنریک: «شاید» — همان `Option` ولی ساختهٔ خودمان.
#[derive(Debug, Clone, PartialEq)]
pub enum Maybe<T> {
    /// چیزی هست
    Just(T),
    /// چیزی نیست
    Nothing,
}

impl<T> Maybe<T> {
    /// تبدیل به `Option` استاندارد (تا بشود از `?` استفاده کرد)
    pub fn ok(self) -> Option<T> {
        match self {
            Maybe::Just(v) => Some(v),
            Maybe::Nothing => None,
        }
    }

    /// نگاشت مقدار داخل (مثل `Option::map`)
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Maybe<U> {
        match self {
            Maybe::Just(v) => Maybe::Just(f(v)),
            Maybe::Nothing => Maybe::Nothing,
        }
    }
}

fn generic_types() {
    let mut s: Stack<i32> = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    println!("  پشتهٔ i32: len={} peek={:?}", s.len(), s.peek());
    s.print_top_down();
    println!("  pop → {:?} و بعد peek → {:?}", s.pop(), s.peek());

    let mut names: Stack<String> = Stack::new();
    names.push("زهرا".into());
    names.push("علی".into());
    names.print_top_down();

    // نوع استنتاج می‌شود؛ نیازی به نوشتن Stack::<String> نیست
    let m: Maybe<i32> = Maybe::Just(21);
    println!("  Maybe: {:?} → map(x*2) = {:?}", m.clone(), m.map(|x| x * 2));
    println!("  ok() → {:?}", Maybe::Just(5).ok());
}

// ═══════════════════════════════════════════════════════════════════════════
/// صفت «ظرف» با **نوع وابسته** (associated type).
///
/// تفاوت کلیدی با `trait Container<T>`:
/// * نوع وابسته → هر پیاده‌سازی **یک** نوع عنصر دارد (مثل Iterator).
/// * پارامتر ژنریک → یک نوع می‌تواند چند پیاده‌سازی داشته باشد.
trait Container {
    /// نوع عنصر داخل ظرف
    type Item;

    /// طول
    fn len(&self) -> usize;

    /// آیا خالی است؟ (متد پیش‌فرض → پیاده‌سازها لازم نیست تکرارش کنند)
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// از نوع وابسته در امضا استفاده می‌کنیم
    fn first(&self) -> Option<&Self::Item>;
}

impl<T> Container for Vec<T> {
    type Item = T;
    fn len(&self) -> usize {
        Vec::len(self)
    }
    fn first(&self) -> Option<&T> {
        // توجه: باید `<[T]>::first` بنویسیم، وگرنه متد *خودِ صفت* را صدا می‌زنیم
        // و بازگشت بی‌پایان (unconditional recursion) می‌شود!
        <[T]>::first(self)
    }
}

impl<T> Container for std::collections::VecDeque<T> {
    type Item = T;
    fn len(&self) -> usize {
        std::collections::VecDeque::len(self)
    }
    fn first(&self) -> Option<&T> {
        self.front()
    }
}

impl<T, const N: usize> Container for [T; N] {
    // 🎯 «const generic» — آرایه با طول N که در زمان کامپایل معلوم است
    type Item = T;
    fn len(&self) -> usize {
        N
    }
    fn first(&self) -> Option<&T> {
        <[T]>::first(self)
    }
}

/// تابعی که هر ظرفی را می‌پذیرد — با نوع وابسته.
fn summarize<C: Container>(c: &C) -> String {
    match c.first() {
        Some(_) => format!("{} عنصر (خالی نیست)", c.len()),
        None => "خالی".to_string(),
    }
}

fn associated_types() {
    println!("  Vec       → {}", summarize(&vec![1, 2, 3]));
    println!("  VecDeque  → {}", summarize(&std::collections::VecDeque::from([1, 2])));
    println!("  آرایه [؛۳] → {}", summarize(&[9, 8, 7]));
    println!("  [i32;0]   → {}", summarize::<[i32; 0]>(&[]));
    println!("  is_empty برای Vec: {}", vec![0; 0].is_empty());
    println!("  ترفند: متد پیش‌فرض `is_empty` را هر پیاده‌سازی رایگان می‌گیرد");
}

// ═══════════════════════════════════════════════════════════════════════════
/// بردار دوبعدی — برای نشان دادن سربارگذاری اپراتور.
///
/// `#[repr(C)]` می‌گوید «ترتیب فیلدها مثل C، بدون بهینه‌سازی چیدمان».
/// این تضمین لازم است تا بتوانیم در `Deref` با اطمینان به `(f64, f64)` نگاه کنیم.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    /// مختصهٔ افقی
    pub x: f64,
    /// مختصهٔ عمودی
    pub y: f64,
}

impl Vector2 {
    /// سازنده
    pub fn new(x: f64, y: f64) -> Self {
        Vector2 { x, y }
    }

    /// طول برداری (اندازه)
    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// `+` را سربارگذاری می‌کنیم
impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

// `+=`
impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// `*` با عدد (ضرب اسکالر)
impl Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f64) -> Vector2 {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}

// درآیندکردن (`*v`) و ایندکس (`v[0]`) هم اپراتورهای قابل سربارگذاری‌اند
impl Index<usize> for Vector2 {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("ایندکس {i} خارج از محدودهٔ Vector2"),
        }
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

// Deref: «همان‌طور که یک &T را استفاده می‌کنی، این را هم استفاده کن».
// ⚠️ قانون طراحی: Deref را فقط برای انواع هوشمند (smart pointer) پیاده کن!
// برای پوششی‌های منطقی (newtype) این کار بدعت است، چون متدهای پنهان اضافه می‌کند.
impl Deref for Vector2 {
    type Target = (f64, f64);
    fn deref(&self) -> &Self::Target {
        // با `#[repr(C)]` می‌دانیم `(f64, f64)` دقیقاً همین چیدمان را دارد،
        // پس این تبدیل درست است. (در کد تولیدی راه ساده‌تر: یک فیلد tuple نگه دار
        // و `deref` را به آن بده — بدون هیچ unsafe. اینجا برای نمایش توانایی است.)
        unsafe { &*(self as *const Vector2 as *const (f64, f64)) }
    }
}

fn operator_overloading() {
    let a = Vector2::new(1.0, 2.0);
    let b = Vector2::new(3.0, -1.0);

    println!("  a = {a}، b = {b}");
    println!("  a + b = {}", a + b);
    println!("  a * 2 = {}", a * 2.0);
    println!("  |a| = {:.3}", a.magnitude());
    println!("  a[0]={}، a[1]={}", a[0], a[1]);

    let mut c = a;
    c += b;
    println!("  a += b → {c}");

    // Deref در عمل: متدهای tuple خودکار در دسترس‌اند
    println!("  با Deref: c.0={}، c.1={}", c.0, c.1);
    println!("  (tuple فقط فیلد .0/.1 دارد؛ Deref یعنی فیلدهای آن مستقیم در دسترس‌اند)");
    println!("  ⚠️ هشدار طراحی: Deref روی انواع غیراشاره‌گر = افزودن رایگان متدهای ناخواسته");
}

// ═══════════════════════════════════════════════════════════════════════════
/// کد پستی معتبر (نمونهٔ «Newtype با اعتبارسنجی»).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalCode(String);

/// خطای اعتبارسنجی کد پستی.
#[derive(Debug, PartialEq)]
pub enum PostalCodeError {
    /// طول اشتباه
    WrongLength(usize),
    /// کاراکتر غیرمجاز
    BadChar(char),
}

impl fmt::Display for PostalCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostalCodeError::WrongLength(n) => write!(f, "طول کد پستی باید ۱۰ رقم باشد، ولی {n} بود"),
            PostalCodeError::BadChar(c) => write!(f, "کاراکتر غیرمجاز «{c}» در کد پستی"),
        }
    }
}

impl std::error::Error for PostalCodeError {}

impl TryFrom<&str> for PostalCode {
    type Error = PostalCodeError;

    /// 🛡️ نکتهٔ کلیدی: اگر این نوع ساخته شد، **قطعاً معتبر است**.
    /// دیگر هیچ‌جای برنامه لازم نیست دوباره اعتبارسنجی کنی (Parse, don't validate).
    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        let digits: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
        for c in digits.chars() {
            if !c.is_ascii_digit() {
                return Err(PostalCodeError::BadChar(c));
            }
        }
        if digits.chars().count() != 10 {
            return Err(PostalCodeError::WrongLength(digits.chars().count()));
        }
        Ok(PostalCode(digits))
    }
}

impl fmt::Display for PostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1234567890 → «12345-67890»
        let d: Vec<char> = self.0.chars().collect();
        write!(f, "{}-{}", d[..5].iter().collect::<String>(), d[5..].iter().collect::<String>())
    }
}

fn conversions() {
    // TryFrom ترجیح داده می‌شود چون شکست می‌تواند داشته باشد
    match PostalCode::try_from("۱۲۳۴۵۶۷۸۹۰") {
        Ok(pc) => println!("  درست: {pc}"),
        Err(e) => println!("  خطا (همان‌طور که باید): {e}"),
    }
    match PostalCode::try_from("1234567890") {
        Ok(pc) => println!("  درست: {pc}"),
        Err(e) => println!("  خطا: {e}"),
    }
    // ترفند: `try_into()` شکل ارگونومیک `TryFrom` است
    let res: Result<PostalCode, _> = "99999 88888".try_into();
    println!("  با try_into: {res:?}");

    // From برای تبدیل‌های بی‌خطا: خودکار `Into` هم می‌سازد
    struct Celsius(f64);
    impl From<f64> for Celsius {
        fn from(v: f64) -> Self {
            Celsius(v)
        }
    }
    impl From<Celsius> for f64 {
        fn from(c: Celsius) -> f64 {
            c.0
        }
    }
    let c: Celsius = 36.6.into();
    let back: f64 = c.into();
    println!("  From/Into رفت و برگشت: {back}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn modern_generics() {
    // ۱) `impl Trait` در خروجی = «یک نوع مشخص، ولی اسمش مهم نیست»
    fn evens(n: usize) -> impl Iterator<Item = usize> {
        (0..n).filter(|x| x % 2 == 0)
    }
    println!("  impl Iterator در خروجی: {:?}", evens(10).collect::<Vec<_>>());

    // ۲) const generics — طول آرایه در زمان کامپایل
    #[derive(Debug)]
    struct Buffer<const N: usize> {
        data: [u8; N],
    }
    impl<const N: usize> Buffer<N> {
        fn new() -> Self {
            Buffer { data: [0; N] }
        }
        fn size(&self) -> usize {
            N
        }
    }
    let b4 = Buffer::<4>::new();
    let b1024 = Buffer::<1024>::new();
    println!("  Buffer<4>.size()={} Buffer<1024>.size()={}", b4.size(), b1024.size());
    println!("  هر دو در زمان کامپایل با طول متفاوت ساخته شده‌اند (بدون تخصیص heap)");

    // ۳) نمونه‌ای از واقعیت: تابعی که آرایهٔ هر طولی را می‌پذیرد
    fn sum_all<const N: usize>(arr: [i64; N]) -> i64 {
        arr.iter().sum()
    }
    println!("  sum_all([1,2,3])={} sum_all([1,2,3,4])={}", sum_all([1, 2, 3]), sum_all([1, 2, 3, 4]));

    // ۴) 🧠 تک‌ریخت‌سازی: هر T یک نسخهٔ مستقل تولید می‌کند.
    //    (اگه در این فایل چند نوع استفاده کنی، باینری چند نسخه دارد)
    println!("  توجه: کد ژنریک در زمان کامپایل «تخصصی» می‌شود → هزینهٔ صفر در اجرا");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn stack_works() {
        let mut s = Stack::new();
        s.push(1);
        s.push(2);
        assert_eq!(s.peek(), Some(&2));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
    }

    #[test]
    fn largest_finds_max() {
        assert_eq!(largest_copy(&[3, 9, 1]), Some(9));
        assert_eq!(largest_copy::<i32>(&[]), None);
        assert_eq!(largest_copy(&["ب", "الف"]), Some("ب"));
    }

    #[test]
    fn operator_overloading_is_correct() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(3.0, 4.0);
        assert_eq!(a + b, Vector2::new(4.0, 6.0));
        assert_eq!(a * 2.0, Vector2::new(2.0, 4.0));
        assert_eq!(a[1], 2.0);
        assert!((Vector2::new(3.0, 4.0).magnitude() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn postal_code_validates() {
        assert!(PostalCode::try_from("12345 67890").is_ok());
        assert_eq!(PostalCode::try_from("123").unwrap_err(), PostalCodeError::WrongLength(3));
        assert_eq!(PostalCode::try_from("abcdefghij").unwrap_err(), PostalCodeError::BadChar('a'));
        assert_eq!(PostalCode::try_from("1234567890").unwrap().to_string(), "12345-67890");
    }

    #[test]
    fn container_trait_is_generic_over_types() {
        assert_eq!(summarize(&vec![1, 2]), "2 عنصر (خالی نیست)");
        assert_eq!(summarize(&[0u8; 0]), "خالی");
    }

    #[test]
    fn maybe_behaves_like_option() {
        assert_eq!(Maybe::Just(2).map(|x| x + 1), Maybe::Just(3));
        assert_eq!(Maybe::<i32>::Nothing.map(|x| x + 1), Maybe::Nothing);
        assert_eq!(Maybe::Just(1).ok(), Some(1));
    }
}
