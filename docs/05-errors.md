# ⚠️ ۰۵ — مدیریت خطا در Rust

## ۱) فلسفه: خطا یک مقدار است

```rust
fn divide(a: f64, b: f64) -> Result<f64, MathError> { … }
```
* **خطای قابل بازیابی** (ورودی بد، فایل نیست) → `Result<T, E>`
* **باگ برنامه‌نویس** (فرض نقض‌شده) → `panic!`
* **نمونه‌سازی سریع** (اسکریپت، تست) → `unwrap()`

قاعدهٔ آهنین: **کتابخانه هرگز نباید پنیک کند** (مگر شرط‌هایش را در مستندات
`# Panics` گفته باشد).

---

## ۲) ابزارهای پایه

```rust
// ساخت
Ok(value)
Err(MyError::Bad)
Err("متن".to_string())

// از Option به Result
opt.ok_or(MyError::Missing)?          // خطای ثابت
opt.ok_or_else(|| MyError::new(x))?   // خطای محاسبه‌شده (تنبل)

// از Result به Option
res.ok()          // خطا را دور می‌ریزد
res.err()         // مقدار را دور می‌ریزد

// مقادیر پیش‌فرض
res.unwrap_or(0)
res.unwrap_or_default()
res.unwrap_or_else(|_| compute())

// پنیک‌کردن (فقط وقتی باگ است)
res.unwrap()                      // ❌ پیام بی‌فایده
res.expect("فایل پیکربندی باید وجود داشته باشد")  // ✅ قابل‌فهم
```

---

## ۳) عملگر `?`

```rust
fn read_config() -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string("app.toml")?;   // io::Error → ConfigError (با From)
    let port = text.trim().parse()?;                    // ParseIntError → ConfigError
    Ok(Config { port })
}
```
معادل واقعی‌اش:
```rust
let text = match std::fs::read_to_string("app.toml") {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),
};
```

### گزینه‌های مدرن `?`
```rust
// روی Option (در تابعی که Option برمی‌گرداند)
let x = vec.first()?;

// تبدیل در محل با ?
let v: i32 = s.parse().map_err(|_| MyError::BadNumber)?;

// در Rust 1.65+: `let-else` برای زودهنگام برگرداندن بدون Result
let Some(user) = users.get(1) else { return Err(Error::NotFound) };
```

⚠️ **`?` روی `Result` در تابعی که `Option` برمی‌گرداند کار نمی‌کند** — `.ok()?`
لازم است (یا برگرداندن `Result`).

---

## ۴) خطای سفارشی — الگوی حرفه‌ای

```rust
#[derive(Debug)]
pub enum AppError {
    Io { ctx: String, src: std::io::Error },   // ← «زمینه» + علت
    Json(crate::json::JsonError),
    NotFound(u64),
    BadInput(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io { ctx, src } => write!(f, "خطای «{ctx}»: {src}"),
            AppError::NotFound(id) => write!(f, "شناسهٔ {id} پیدا نشد"),
            …
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io { src, .. } => Some(src),   // ← زنجیرهٔ علت
            AppError::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {         // ← ? خودکار تبدیل می‌کند
    fn from(src: std::io::Error) -> Self { AppError::Io { ctx: "I/O".into(), src } }
}
```

### سه قاعدهٔ طراحی خطا
1. **`enum` برای کتابخانه** (کاربر می‌تواند با `match` تشخیص دهد).
2. **`Box<dyn Error>` برای برنامه** (همه‌چیز را می‌پذیرد، سادگی).
3. **همیشه `Display` + `Error` پیاده کن** (سازگاری با اکوسیستم).

### کد خروج (exit code) — برای CLI
```rust
impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Usage(_) | AppError::BadInput(_) => 2,   // استفادهٔ نادرست
            AppError::TaskNotFound(_) => 3,                    // نبود منبع
            _ => 1,                                            // خطای عمومی
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => { eprintln!("✖ {e}"); ExitCode::from(e.exit_code() as u8) }
    }
}
```

---

## ۵) چاپ زنجیرهٔ خطا (مثل anyhow)

```rust
eprintln!("✖ {err}");
let mut src = std::error::Error::source(&err);
while let Some(cause) = src {
    eprintln!("  ↳ {cause}");
    src = cause.source();
}
```
خروجی:
```text
✖ خطای «خواندن فایل data/tasks.json»: No such file or directory
  ↳ No such file or directory (os error 2)
```

---

## ۶) ترکیب‌کننده‌ها به‌جای `match` زنجیره‌ای

| متد | کار |
|-----|-----|
| `.map(f)` | تبدیل مقدار موفق |
| `.map_err(f)` | تبدیل خطا |
| `.and_then(f)` | زنجیرهٔ مرحله‌های خطاپذیر |
| `.or_else(f)` | جایگزین روی خطا |
| `.unwrap_or(x)` / `.unwrap_or_else(f)` | پیش‌فرض |
| `.inspect(f)` | نگاه جانبی (دیباگ)، بدون تغییر |
| `.is_ok_and(f)` / `.is_err_and(f)` | بررسی شرطی |
| `.ok()` / `.err()` | تبدیل به Option |
| `.transpose()` | `Option<Result>` ↔ `Result<Option>` |

```rust
let addr = parse_port("8080").and_then(|p| parse_host(p)).and_then(build_addr);
```
🎯 هر کم‌کردن یک `match` تودرتو، خوانایی را بالا می‌برد.

---

## ۷) `collect` روی `Result` — الگوی طلایی

```rust
let numbers: Result<Vec<i32>, _> = ["1", "2", "3"].iter().map(|s| s.parse()).collect();
// یا همه موفق، یا اولین خطا (اتمیک)
```
و اگر می‌خواهی **همهٔ** خطاها را بگیری:
```rust
let (oks, errs): (Vec<_>, Vec<_>) = inputs.iter().map(|s| s.parse::<i32>()).partition(Result::is_ok);
```

---

## ۸) پنیک — کجا درست است؟

| موقعیت | پنیک درست؟ |
|--------|-----------|
| `assert!` در تست | ✅ |
| نقض invariant درونی (`unreachable!`) | ✅ |
| ایندکس خارج از محدوده در کد خودت | ⚠️ بهتر است `get` |
| ورودی کاربر | ❌ هرگز |
| خطای شبکه/فایل | ❌ هرگز |
| داخل `Drop` | ❌ هرگز (پنیک در پنیک = مرگ فرایند) |
| `unwrap()` روی `Mutex::lock()` | ⚠️ مسمومیت ممکن است → `unwrap_or_else(\|e\| e.into_inner())` |

```rust
panic!("پیام");                       // با فرمت هم می‌شود
unreachable!("شرط باید همیشه برقرار باشد");
todo!("بعداً پیاده می‌کنم");            // ✅ نوع درست برمی‌گرداند، کد کامپایل می‌شود
unimplemented!();
debug_assert!(x > 0);                  // ← در release حذف می‌شود (هزینهٔ صفر)
assert_eq!(a, b, "پیام با مقدار {a}");
```
گرفتن پنیک (فقط در مرزهای بحرانی مثل ترد/FFI):
```rust
let result = std::panic::catch_unwind(|| { … });
```

---

## ۹) الگوهای حرفه‌ای

### الف) «پارس کن، اعتبارسنجی نکن»
```rust
pub struct Email(String);
impl TryFrom<&str> for Email {
    type Error = EmailError;
    fn try_from(raw: &str) -> Result<Self, Self::Error> { … }
}
// حالا هر جا Email داری، قطعاً معتبر است — دیگر چک دوباره لازم نیست.
```

### ب) تفکیک خطای موقت از دائمی
```rust
enum FetchError { Timeout, NotFound, Parse(String) }
impl FetchError {
    fn is_retryable(&self) -> bool { matches!(self, FetchError::Timeout) }
}
```

### ج) خطا با «زمینه» در هر لایه
```rust
.map_err(|e| AppError::io(format!("خواندن «{}»", path.display()), e))?;
// پیام نهایی: خطای «خواندن «data/tasks.json»»: Permission denied
```

### د) `main` با `Result` (برای برنامه‌های کوچک)
```rust
fn main() -> Result<(), Box<dyn Error>> {
    let text = std::fs::read_to_string("f.txt")?;    // ? آزادانه کار می‌کند
    println!("{text}");
    Ok(())
}
```

### ه) اکوسیستم
| crate | برای |
|-------|------|
| `thiserror` | تولید `Display`/`Error`/`From` برای enum خطا (کتابخانه) |
| `anyhow` | خطای همه‌کاره + `context()` + backtrace (برنامه) |
| `eyre` | مثل anyhow با گزارش‌های زیبا |

```rust
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("فایل {0} پیدا نشد")]
    Missing(String),
    #[error("خطای شبکه")]
    Network(#[from] std::io::Error),      // ← From خودکار
}
```

---

## ۱۰) اشتباهات رایج

| اشتباه | درست |
|--------|------|
| `.unwrap()` در مسیر کاربر | `?` یا `unwrap_or_else` با پیام |
| `Box<dyn Error>` در API کتابخانه | `enum` تایپ‌دار |
| خطای بدون `source()` | زنجیرهٔ علت را نگه دار |
| `map_err(\|_\| MyError)` (علت گم می‌شود) | علت را *داخل* خطا نگه دار |
| پنیک در `Drop` | لاگ کن یا بی‌صدا رد شو |
| `?` در تابعی که نوع خطایش `()` است | `()` پیاده‌سازی `Error` ندارد → `Box<dyn Error>` |
| نادیده‌گرفتن `Result` با `let _ =` | حداقل `if let Err(e) = … { log(e) }` |

[⬅️ قبلی: انواع و صفت‌ها](04-types-and-traits.md) · [بعدی: ایتراتور و بستار ➡️](06-iterators-and-closures.md)
