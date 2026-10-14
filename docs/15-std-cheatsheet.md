# 📋 ۱۵ — چیت‌شیت استاندارد Rust

> مرجع سریع ماژول‌ها و APIهای `std` که هر روز لازم می‌شوند. (این پروژه صفر
> وابستگی دارد — هر چیزی که استفاده کرده، از همین‌هاست.)

---

## ۱) نقشهٔ ماژول‌های `std`

| ماژول | چه دارد |
|-------|---------|
| `std::env` | `args()`, `var()`, `current_dir()`, `set_current_dir()`, `vars()` |
| `std::fs` | `read/write/read_to_string`, `File`, `create_dir_all`, `remove_file`, `rename`, `metadata`, `read_dir` |
| `std::path` | `Path`, `PathBuf`, `Component`, `canonicalize` |
| `std::io` | `stdin/stdout/stderr`, `BufRead`, `Write`, `ErrorKind`, `Cursor` |
| `std::process` | `Command`, `ExitCode`, `exit`, `Stdio` |
| `std::collections` | `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `VecDeque`, `BinaryHeap`, `LinkedList` |
| `std::vec` / `std::string` | `Vec`, `String`, `FromIterator` |
| `std::str` / `std::char` | `Split`, `Lines`, `Chars`, `from_utf8`, `char::from_u32` |
| `std::fmt` | `Display`, `Debug`, `Write` (ماکرو `write!`), `format!` |
| `std::iter` | `Iterator`, `IntoIterator`, `FromIterator`, `Peekable`, `Chain`, `Enumerate` |
| `std::option` / `std::result` | `Option`, `Result`, `?` |
| `std::time` | `Instant`, `Duration`, `SystemTime`, `UNIX_EPOCH` |
| `std::thread` | `spawn`, `scope`, `sleep`, `JoinHandle`, `available_parallelism` |
| `std::sync` | `Arc`, `Mutex`, `RwLock`, `mpsc`, `OnceLock`, `LazyLock`, `Barrier`, `Condvar` |
| `std::sync::atomic` | `AtomicBool/Usize/…`, `Ordering`, `fence` |
| `std::cell` | `Cell`, `RefCell`, `UnsafeCell`, `OnceCell` |
| `std::rc` | `Rc`, `Weak` |
| `std::borrow` | `Cow`, `Borrow`, `ToOwned` |
| `std::marker` | `PhantomData`, `Send`, `Sync`, `Sized`, `Unpin` |
| `std::mem` | `size_of`, `swap`, `take`, `replace`, `drop`, `forget`, `transmute`, `ManuallyDrop` |
| `std::ptr` | `null`, `addr`, `read`, `write`, `copy`, `NonNull` |
| `std::ffi` | `CStr`, `CString`, `OsStr`, `OsString`, `c_int`, `c_char` |
| `std::cmp` | `Ordering`, `min`, `max`, `Reverse` |
| `std::num` | `NonZeroU32`, `Saturating`, `Wrapping`, `parse::<T>()` |
| `std::error` | `Error` trait, `Box<dyn Error>` |
| `std::any` | `Any`, `TypeId`, `downcast_ref` |
| `std::panic` | `catch_unwind`, `set_hook`, `panic::Location` |
| `std::hash` | `Hash`, `Hasher`, `DefaultHasher` |
| `std::net` | `TcpListener`, `TcpStream`, `UdpSocket`, `SocketAddr` |
| `std::array` | `from_fn`, `IntoIter`, `map` (const) |
| `std::slice` | `sort`, `chunks`, `windows`, `binary_search`, `split_at_mut` |

---

## ۲) `Option` / `Result` — تمام متدها

```rust
// Option
opt.is_some() / is_none()
opt.unwrap() / expect("msg") / unwrap_or(x) / unwrap_or_default() / unwrap_or_else(f)
opt.map(f) / map_or(default, f) / and_then(f) / or(other) / or_else(f) / filter(p)
opt.ok_or(err) / ok_or_else(f)        // → Result
opt.take()                             // خالیاش میکند و مقدار را میدهد
opt.as_ref() / as_mut()                // → Option<&T>
opt.zip(other) / flatten() / replace(v) / get_or_insert(v)
opt.iter()                             // 0 یا 1 عضو
opt?                                   // با ? در تابع Option

// Result
res.is_ok() / is_err()
res.ok() / err()                       // → Option
res.map(f) / map_err(g) / and_then(f) / or_else(f)
res.unwrap_or(x) / unwrap_or_else(f) / unwrap_or_default()
res.expect("…")
res.as_ref() / as_mut()
res.iter()
res?                                   // با ? در تابع Result
```

```rust
// 🔑 جدول تبدیلها
opt.ok_or("err")?                      // Option → Result + ?
res.ok()                               // Result → Option
res.unwrap_or_default()                // پیشفرض نوع
Option::<T>::from(x) / FromIterator
```

---

## ۳) `Iterator` — همه متدها

**سازنده (Adapter، تنبل):**
`map` `filter` `filter_map` `flat_map` `flatten` `take` `take_while`
`skip` `skip_while` `step_by` `chain` `zip` `enumerate` `inspect` `peekable`
`cycle` `rev` `cloned` `copied` `by_ref` `scan` `fuse` `array_chunks`(nightly)

**مصرف‌کننده (Consumer):**
`collect` `fold` `try_fold` `reduce` `sum` `product` `count` `last` `max`
`min` `max_by` `min_by` `max_by_key` `min_by_key` `find` `find_map` `position`
`any` `all` `for_each` `try_for_each` `partition` `unzip` `nth` `next`
`size_hint` `sum` `cmp`

```rust
// نمونههای کاربردی
let names: Vec<&str> = tasks.iter().filter(|t| !t.done).map(|t| t.title.as_str()).collect();
let max_id = tasks.iter().map(|t| t.id.get()).max().unwrap_or(0);
let pct = done as f64 / total as f64 * 100.0;
let (done, pending): (Vec<_>, Vec<_>) = tasks.iter().partition(|t| t.done);
let by_priority: HashMap<Priority, usize> =
    tasks.iter().fold(HashMap::new(), |mut m, t| { *m.entry(t.priority).or_insert(0) += 1; m });
```

---

## ۴) `Vec` / `String` — روش‌های پرکاربرد

```rust
// Vec
v.push(x) / pop() / insert(i, x) / remove(i) / swap_remove(i)
v.get(i) / get_mut(i) / first() / last()
v.len() / is_empty() / capacity() / shrink_to_fit() / reserve(n) / clear()
v.iter() / iter_mut() / into_iter() / drain(a..b) / split_at_mut(i) / split_off(i)
v.windows(n) / chunks(n) / chunks_exact(n) / sort() / sort_by() / sort_unstable()
v.dedup() / dedup_by_key() / retain(p) / truncate(n) / extend(iter) / extend_from_slice(s)
v.contains(&x) / binary_search(&x) / concat() / join(sep) / flatten() (روی Vec<Vec>)
vec![0; 100] / Vec::with_capacity(100) / v.as_slice() / v.into_boxed_slice()

// String
String::new() / with_capacity(n) / from("x") / from_utf8(bytes) / from_utf8_lossy(&b)
s.push('c') / push_str("x") / insert(i, 'c') / pop() / truncate(n) / clear()
s.len() (بایت) / chars().count() (کاراکتر) / is_empty() / capacity()
s.as_str() / as_bytes() / into_bytes() / to_string() / clone()
s.split(sep) / splitn(n, sep) / rsplit / lines() / split_whitespace() / split_terminator()
s.trim() / trim_start() / trim_end() / trim_matches(chars) / strip_prefix(p) / strip_suffix(p)
s.starts_with(p) / ends_with(p) / contains(p) / find(p) / rfind(p) / replace(a, b) / replacen
s.to_lowercase() / to_uppercase() / to_ascii_lowercase()
s.parse::<i32>() / format!("{s}") / s.get(a..b) / s.char_indices() / s.is_char_boundary(i)
```

---

## ۵) کلکسیون‌ها

```rust
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, BinaryHeap};

// HashMap
map.insert(k, v) / get(&k) → Option<&V> / get_mut(&k) / remove(&k) / contains_key(&k)
map.entry(k).or_insert(v) / .or_insert_with(f) / .or_default() / .and_modify(f)
map.keys() / values() / values_mut() / iter() / len() / is_empty() / clear()
map.retain(|k, v| …) / extend(iter)
// ترتیبپذیر:
// BTreeMap: range(1..5) / first_key_value() / pop_first()

// HashSet / BTreeSet
set.insert(v) → bool / contains(&v) / remove(&v)
set.union(&other) / intersection / difference / symmetric_difference / is_subset / is_superset

// VecDeque (صف)
dq.push_back(x) / pop_front() / push_front() / pop_back() / front() / back() / rotate_left(n)

// BinaryHeap (صف اولویت)
heap.push(x) / pop() → Option<T> (بزرگترین) / peek()
// برای «کوچکترین اول»: BinaryHeap<Reverse<T>>
```
🎯 مرتب‌سازی نتیجهٔ `HashMap` برای خروجی پایدار:
```rust
let mut items: Vec<_> = map.into_iter().collect();
items.sort_by(|a, b| b.1.cmp(&a.1));            // بر اساس تعداد نزولی
```

---

## ۶) مسیر و فایل

```rust
use std::path::{Path, PathBuf};

let p = Path::new("data/tasks.json");
p.file_name()          // Option<&OsStr>   → tasks.json
p.file_stem()          // → tasks
p.extension()          // → json
p.parent()             // → data
p.exists() / is_file() / is_dir()
p.join("x") / with_extension("bak") / to_string_lossy() / display()
p.canonicalize()       // مسیر مطلق (فایل باید موجود باشد)

let pb: PathBuf = p.to_path_buf();
pb.push("x"); pb.set_extension("txt");

// فایل
std::fs::read_to_string("f")? / write("f", b"x")? / read("f")?
std::fs::create_dir_all("a/b")? / remove_file("f")? / remove_dir_all("a")?
std::fs::rename("tmp", "final")? / copy(a, b)? / metadata("f")?.len()
for entry in std::fs::read_dir(".")? { let e = entry?; println!("{}", e.path().display()); }
```
🎯 «نوشتن اتمیک» = نوشتن در فایل موقت + `rename`:
```rust
let tmp = path.with_extension("tmp");
std::fs::write(&tmp, json)?;
std::fs::rename(&tmp, path)?;
```

---

## ۷) زمان، تاریخ، عدد

```rust
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();  // unix ثانیه
let t = Instant::now(); t.elapsed();                                // یکنوا
Duration::from_millis(500) / from_secs(2) / as_millis() / as_secs_f64()
Duration::from_secs(90).as_secs()                 // 90

// انسانسازی زمان (کاری که humanize_age ما میکند)
fn humanize(secs: u64) -> String {
    match secs {
        0..=59 => "الان".to_string(),
        60..=3599 => format!("{} دقیقه پیش", secs / 60),
        3600..=86_399 => format!("{} ساعت پیش", secs / 3600),
        d => format!("{} روز پیش", d / 86_400),
    }
}
// تاریخ شمسی؟ → crate «chrono» + «jiff» یا الگوریتم تبدیل دستی.
```

```rust
// عدد
i32::MAX / MIN / checked_add / saturating_mul / wrapping_sub / overflowing_div
f64::NAN / INFINITY / EPSILON / is_nan() / is_finite() / round() / floor() / ceil()
(3.7_f64).round() as i64 / f64::from(i32) / x as f64
"42".parse::<i32>() / "3.14".parse::<f64>()
u32::from_str_radix("ff", 16)          // 255
format!("{:b} {:o} {:x} {:X} {:e}", 10, 10, 255, 255, 1000.0)
```

---

## ۸) `std::env` / `std::process`

```rust
let args: Vec<String> = std::env::args().skip(1).collect();
std::env::var("HOME") / var_os / set_var (unsafe در edition 2024!) / vars()
std::env::current_dir() / set_current_dir(p)
std::env::temp_dir()                     // پوشهٔ موقت
std::env::consts::{OS, ARCH, FAMILY}     // زمان کامپایل

let out = std::process::Command::new("ls").arg("-l").current_dir("/tmp").output()?;
println!("{}", String::from_utf8_lossy(&out.stdout));
std::process::exit(2);
```

---

## ۹) `std::io`

```rust
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};

let mut line = String::new();
io::stdin().read_line(&mut line)?;                  // شامل \n
print!("..."); io::stdout().flush()?;               // 🎯 print! flush نمیکند
eprintln!("خطا");                                    // stderr
write!(io::stderr(), "{msg}")?;                      // بدون تخصیص

let mut f = BufReader::new(std::fs::File::open("big")?);
for line in f.lines() { let l = line?; }

let mut w = BufWriter::new(std::fs::File::create("out")?);
w.write_all("سلام".as_bytes())?;                     // 🎯 write_all نه write

let mut s = String::new();
std::fs::File::open("f")?.read_to_string(&mut s)?;
```

---

## ۱۰) `std::fmt` — هر چیزی که باید بلد باشی

```rust
{}        // Display
{:?}      // Debug
{:#?}     // Debug چندخطی زیبا
{:>8} {:<8} {:^8}      // راست/چپ/وسط با عرض
{:08}     // پرکردن با صفر
{:+}      // علامت اجباری
{:.3}     // دقت اعشار
{:e} {:E} // نمایی
{:b} {:o} {:x} {:X} {:#b} {:#x}
{:#width$} {:.*}       // عرض/دقت از متغیر
{named} {0} {1}        // نامی یا اندیسی
{{ }}                  // آکولاد خالی
```
```rust
// پیادهسازی Display برای نوع خودت
impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
// 🎯 برای چاپ رنگشده، میتوانی padding را رعایت کنی:
write!(f, "{}", self.0)                    // رنگها را در رنگگذاری ماژول color بگذار
```

---

## ۱۱) الگوهای std که در همین پروژه به کار رفته

```rust
// ۱) خواندن داده با پیشفرض امن (env یا فایل)
std::fs::read_to_string(path).unwrap_or_default()

// ۲) زمان حال (unix)
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
// 🎯 `.map(...)` چون duration_since قبل ۱۹۷۰ خطا میدهد

// ۳) مقداردهی تصادفی نیست! برای «شناسهٔ یکتا» از max+1 استفاده میکنیم
let next_id = tasks.iter().map(|t| t.id.get()).max().unwrap_or(0) + 1;

// ۴) پوششدادن خواندن خطوط فرمان
pub fn parse_env() -> Args { Args::parse_from(std::env::args().skip(1)) }

// ۵) نوشتن اتمیک
let tmp = path.with_extension("tmp");
std::fs::write(&tmp, data)?;
std::fs::rename(&tmp, path)?;

// ۶) برش امن روی رشتهٔ فارسی
let head = line.get(..n).unwrap_or(line);

// ۷) فاصلهگذاری اعداد
fn group_digits(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { out.push('٫'); }
        out.push(ch);
    }
    out
}
```

[⬅️ قبلی: کارایی](14-performance.md) · [بعدی: اکوسیستم ➡️](16-ecosystem.md)
