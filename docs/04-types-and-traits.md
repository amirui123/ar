# 🧩 ۰۴ — انواع، ساختارها، صفت‌ها و ژنریک

## ۱) struct — سه شکل

```rust
struct User {                 // ۱) نام‌دار (named fields)
    name: String,
    age: u32,
}

struct Meters(f64);           // ۲) tuple struct — برای newtype

struct Marker;                // ۳) unit struct — بدون داده، مثل نشانگر
```

### متدها و توابع وابسته
```rust
impl User {
    pub fn new(name: impl Into<String>, age: u32) -> Self {   // ← سازندهٔ مرسوم
        Self { name: name.into(), age }
    }
    pub fn name(&self) -> &str { &self.name }                 // ← گتر (قرض می‌دهد)
    pub fn birthday(&mut self) { self.age += 1; }              // ← تغییرپذیر
    pub fn into_name(self) -> String { self.name }             // ← مصرف می‌کند
}
```
🎯 **قرارداد نام‌ها:** `new` (سازنده)، `into_*` (مصرف و تبدیل)، `as_*` (تبدیل ارزان/قرض)،
`to_*` (تبدیل گران/کپی)، `is_*` (پرسش بولین).

### `impl` چندتایی و قیددار
```rust
impl<T> Stack<T> { fn new() -> Self { … } }                 // برای هر نوع
impl<T: fmt::Display> Stack<T> { fn print(&self) { … } }     // فقط برای قابل‌نمایش‌ها
impl Stack<i32> { fn sum(&self) -> i32 { … } }               // فقط برای i32
```

---

## ۲) enum — «یا این، یا آن»

```rust
enum Shape {
    Circle { r: f64 },        // ساختار نام‌دار
    Rect(f64, f64),           // tuple
    Empty,                    // بدون داده
}
```
`enum` در Rust **Algebraic Data Type** است: هر واریانت می‌تواند دادهٔ متفاوتی داشته باشد.
به همین دلیل `Option<T>` و `Result<T, E>` هم enum هستند و همین یک ویژگی،
جایگزین `null` و استثناها می‌شود.

### نکات حرفه‌ای روی enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]     // صفت‌های مرسوم
pub enum Priority { Low, Normal, High, Urgent }

impl Priority {
    pub const ALL: [Priority; 4] = [Self::Low, Self::Normal, Self::High, Self::Urgent];
    pub fn as_str(self) -> &'static str { … }          // ← تبدیل به متن برای ذخیره‌سازی
    pub fn weight(self) -> u32 { … }                   // ← داده‌ی همراه (ترفند: رفتار روی واریانت)
}

impl FromStr for Priority {                            // ← پارس از متن
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { … }
}

impl Default for Priority { fn default() -> Self { Self::Normal } }
// یا مدرن‌تر: #[derive(Default)] + #[default] روی یک واریانت (Rust 1.62+)
```

### جدول انتخاب: enum یا struct یا trait؟
| نیاز | ابزار |
|------|-------|
| «یا این یا آن» با داده‌های متفاوت | `enum` |
| «همهٔ این‌ها با هم» | `struct` |
| «رفتار مشترک، پیاده‌سازی‌های متفاوت» | `trait` |
| مجموعهٔ بستهٔ حالت‌ها (کامپایلر جامعیت را چک کند) | `enum` ✅ |
| مجموعهٔ باز (کاربران می‌توانند اضافه کنند) | `trait` ✅ |

🎯 **قاعده:** اگر فهرست حالت‌ها را *تو* می‌دانی و تغییرش می‌دهی → `enum`.
اگر دیگران باید پیاده‌سازی اضافه کنند → `trait`.

---

## ۳) صفت‌ها (traits)

```rust
pub trait Store {
    fn load(&self) -> Result<Snapshot>;               // اجباری
    fn save(&self, snap: &Snapshot) -> Result<()>;    // اجباری

    fn describe(&self) -> String {                    // متد پیش‌فرض (اختیاری)
        "انبارهٔ ناشناخته".to_string()
    }
}
```

### انواع صفت‌های مهم در std
| صفت | کار | نکته |
|-----|-----|------|
| `Display` | نمایش کاربرپسند `{}` | دستی پیاده کن یا `derive` نمی‌شود |
| `Debug` | نمایش دیباگ `{:?}` | تقریباً همیشه `#[derive(Debug)]` |
| `Clone` / `Copy` | کپی | `Copy` ارزان و بدون منبع |
| `PartialEq`/`Eq` | مقایسه | `Eq` برای کلید `HashMap` لازم است |
| `PartialOrd`/`Ord` | مرتب‌سازی | `Ord` برای `BTreeMap` لازم است |
| `Hash` | کلید در `HashMap` | ⚠️ باید با `Eq` همخوان باشد |
| `Default` | مقدار پیش‌فرض | `..Default::default()` را ممکن می‌کند |
| `From`/`Into` | تبدیل | `From` بنویس، `Into` خودکار می‌آید |
| `TryFrom`/`TryInto` | تبدیل خطاپذیر | برای تبدیل‌های تنگ‌شونده |
| `AsRef`/`AsMut` | قرض هم‌خانواده | `impl AsRef<Path>` ترفند کلاسیک |
| `Iterator`, `IntoIterator` | پیمایش | `IntoIterator` برای `for` |
| `Drop` | آزادسازی | کِی منابع آزاد شوند |
| `Deref`/`DerefMut` | رفتار اشاره‌گر | ⚠️ فقط برای smart pointer |
| `Add`,`Mul`,… | سربارگذاری اپراتور | `Add<Output = T>` |
| `Index` | `a[i]` | خروجی `&T` |
| `Error` | خطای استاندارد | برای `Box<dyn Error>` |

### صفت‌های خودکار (auto traits)
`Send`, `Sync`, `Unpin`, `Sized` — **خودکار** پیاده می‌شوند اگر همهٔ اجزا آن‌ها را
داشته باشند. نمی‌توانی دستی پیاده‌شان کنی (به‌جز با `unsafe impl` برای موارد خاص).
🎯 همین‌ها هستند که در `web.rs` باعث شدند `Box<dyn Store>` را به
`Box<dyn Store + Send>` تغییر بدهیم.

### صفت والد (supertrait) و صفت مهرشده
```rust
trait Named: fmt::Display {          // هر Named باید Display هم باشد
    fn short(&self) -> String;
}

mod private { pub trait Sealed {} }  // ← مهر در ماژول خصوصی
trait Public: private::Sealed { }     // بیرون‌ها نمی‌توانند پیاده کنند
```
مزیت مهر: می‌توانی *بعداً* متد اضافه کنی بدون نگرانی از شکستن کد کاربران.

### پیاده‌سازی همه‌جاگیر (blanket impl)
```rust
impl<T: fmt::Display> ToJsonish for T {
    fn to_jsonish(&self) -> String { format!("\"{}\"", self) }
}
```
⚠️ برگشت‌ناپذیر است: بعداً نمی‌توانی برای نوع خاصی رفتار متفاوت بدهی (تعارض).
در std این الگو زیاد است: `impl<T: Display> ToString for T`.

---

## ۴) ژنریک‌ها

```rust
fn largest<T: PartialOrd + Copy>(list: &[T]) -> Option<T> { … }

// با where (خواناتر وقتی قیدها زیاد است)
fn process<T, U>(t: T, u: U) -> String
where
    T: Clone + fmt::Debug,
    U: Into<String>,
{ … }

// ساختار ژنریک
struct Wrapper<T> { inner: T }
impl<T: fmt::Display> Wrapper<T> { fn show(&self) { println!("{}", self.inner) } }

// const generics — طول آرایه در زمان کامپایل
struct Buffer<const N: usize> { data: [u8; N] }
let b = Buffer::<1024> { data: [0; 1024] };
```

### 🧠 تک‌ریخت‌سازی (monomorphization)
کامپایلر برای هر ترکیب نوعِ *استفاده‌شده* یک نسخهٔ تخصصی می‌سازد:

```rust
fn id<T>(x: T) -> T { x }
id(1i32); id(1.0f64);      // ← دو نسخهٔ مستقل در باینری
```
✅ نتیجه: صفر هزینهٔ اجرا (به‌اندازهٔ کد دستی سریع).
⚠️ هزینه: باینری بزرگ‌تر و زمان کامپایل بیشتر (به همین دلیل `dyn` هم وجود دارد).

### `impl Trait` — ژنریک بی‌نام
```rust
fn args(x: impl Display) { }                  // پارامتر (سینتکس راحت‌تر از <T: Display>)
fn evens(n: usize) -> impl Iterator<Item = usize> {     // خروجی
    (0..n).filter(|x| x % 2 == 0)
}
```
⚠️ `impl Trait` در خروجی = «یک نوع ثابت اما بی‌نام». اگر لازم است کالر نامش را
بداند، از `-> Box<dyn Trait>` یا یک نوع `enum` استفاده کن.

### `dyn Trait` — ارسال پویا
```rust
let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { r: 1.0 }), Box::new(Rect(2.0, 3.0))];
let f: &dyn Fn(i32) -> i32 = &|x| x + 1;
```

| | `impl Trait` / ژنریک | `dyn Trait` |
|---|---|---|
| زمان تصمیم | کامپایل | اجرا (vtable) |
| سرعت | صفر هزینه | یک پرش اشاره‌گر |
| حجم باینری | بزرگ‌تر (تک‌ریخت) | کوچک‌تر (یک نسخه) |
| مجموعهٔ ناهمگون | ❌ | ✅ |
| قابلیت نوشتن در فیلد | با پارامتر نوع | مستقیم ✅ |

**قاعدهٔ سرانگشتی:** API کتابخانه → استاتیک؛ مجموعهٔ ناهمگون/Callback → پویا.

### محدودیت‌های `dyn` (object safety)
صفت باید «سازگار با شیء» باشد:
* ❌ متدی که `Self` را برمی‌گرداند یا `Self` را به‌عنوان پارامتر می‌گیرد
* ❌ متد با `where Self: Sized` (این‌ها در vtable نیستند، ولی *می‌توانند* وجود داشته باشند)
* ❌ ژنریک در متد (کامپایلر نمی‌تواند برای همه بی‌نهایت نوع، vtable بچیند)

---

## ۵) تبدیل‌ها — جدول کامل

| می‌خواهی | بنویس |
|----------|-------|
| تبدیل بی‌خطا | `impl From<A> for B` (و `Into` خودکار می‌آید) |
| تبدیل خطاپذیر | `impl TryFrom<A> for B with Error = E` |
| تبدیل قرضی ارزان | `impl AsRef<T>` / `impl Borrow<T>` |
| تبدیل با مصرف | متد `into_*` |
| تبدیل با کپی | متد `to_*` |
| نمایش متن | `impl Display` یا `impl ToString` (از Display می‌آید) |
| پارس از متن | `impl FromStr` |

```rust
impl From<io::Error> for AppError {          // ← همین باعث می‌شود `?` خودکار تبدیل کند
    fn from(e: io::Error) -> Self { AppError::Io { ctx: "…".into(), src: e } }
}
```

🎯 **ترفند ارگونومی در پارامترها:**
```rust
fn new(path: impl Into<PathBuf>) -> Self
fn add(title: impl Into<String>, tags: impl IntoIterator<Item = impl Into<String>>) -> …
```

### 🛡️ Newtype — ترفند ایمنی نوع
```rust
#[repr(transparent)]                    // صفر هزینهٔ اجرا
struct TaskId(u64);

fn complete(user: u64, task: TaskId) { }
complete(TaskId(7), 9);   // ❌ خطای کامپایل — جلوی جابه‌جایی آرگومان‌ها را می‌گیرد
```

### قاعدهٔ یتیم (orphan rule)
نمی‌توانی `impl Display for Vec<f64>` بنویسی (هم صفت هم نوع، بیرونی‌اند).
راه‌حل: newtype (`struct Wrapper(Vec<f64>)`) یا صفت الحاقی برای نوع بیرونی
(`trait MyExt for Vec<f64>` که مجاز است چون صفت مال توست).

---

## ۶) الگوهای طراحی رایج در Rust

| الگو | کِی | چطور |
|------|-----|------|
| **Newtype** | وضوح نوعی، اعتبارسنجی | `struct Email(String)` + `TryFrom<&str>` |
| **Type state** | «وضعیت در نوع» | `struct Connection<State>` با enum‌های `Opened`/`Closed` |
| **Builder** | ساخت مقدار با فیلدهای زیاد | `Foo::builder().a(1).b(2).build()` |
| **Parse, don't validate** | ورودی غیرقابل‌اعتماد | در سازنده معتبر کن، نوع «معتبر» بساز |
| **Newtype با Deref** | پوشش بی‌دردسر | ⚠️ فقط برای اشاره‌گرها |
| **Strategy با trait** | رفتار قابل تزریق | `Box<dyn Fn(String) -> String>` |
| **Enum به‌جای bool** | خوانایی | `enum Status { Active, Inactive }` به‌جای `is_active: bool` |
| **Marker trait** | محدودسازی | `trait ReadOnly {}` |

---

## ۷) نکات ریز که وقت زیاد ذخیره می‌کند

* `#[derive(PartialEq)]` برای `f64` کار می‌کند، ولی `Eq` نه (چون `NaN`).
* برای کلید `HashMap`: `Eq + Hash` لازم است. `f64` هیچ‌کدام را ندارد.
* `#[non_exhaustive]` روی `enum` عمومی = کاربر باید `_ =>` بگذارد (سازگاری نسخه).
* `Self` در `impl` = نوع خودِ impl؛ در صفت = نوع پیاده‌ساز.
* صفت نمی‌تواند فیلد داشته باشد (متفاوت از کلاس انتزاعی). داده در ساختار بگذار.
* `where Self: Sized` متدی را از vtable بیرون می‌آورد (مفید برای `dyn`).
* `impl Trait for &T` و `impl Trait for &mut T` برای ترکیب‌پذیری خوب است
  (مثل `impl Read for &[u8]` در std).

[⬅️ قبلی: مالکیت](03-ownership.md) · [بعدی: خطاها ➡️](05-errors.md)
