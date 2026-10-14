# 📦 ۰۷ — اشاره‌گرهای هوشمند و تغییرپذیری درونی

## ۱) جدول انتخاب

| نوع | مالکیت | تغییرپذیری | اشتراک | تردها | کاربرد |
|-----|--------|------------|--------|-------|--------|
| `Box<T>` | یکتا | عادی | ❌ | اگر T: Send | نوع بازگشتی، اندازهٔ بزرگ، `dyn` |
| `Rc<T>` | چندگانه | ❌ | ✅ | ❌ | گراف/درخت مشترک در یک ترد |
| `Arc<T>` | چندگانه | ❌ | ✅ | ✅ | اشتراک بین تردها |
| `RefCell<T>` | یکتا | ✅ (زمان اجرا) | ❌ | ❌ | تغییرپذیری درونی |
| `Cell<T>` | یکتا | ✅ (کپی) | ❌ | ❌ | فلگ‌های Copy |
| `Mutex<T>` | یکتا | ✅ (قفل) | ✅ | ✅ | وضعیت مشترک در تردها |
| `RwLock<T>` | یکتا | ✅ | ✅ | ✅ | خواندن‌های موازی زیاد |
| `Cow<'a, T>` | هر دو | — | — | — | قرض یا مالکیت، هرکدام ارزان‌تر |
| `Weak<T>` | غیرمالک | — | ✅ | با Arc ✅ | شکستن چرخهٔ حافظه |

---

## ۲) `Box<T>` — ساده‌ترین

```rust
// ۱) نوع بازگشتی (حل اندازهٔ نامشخص)
enum List { Cons(i32, Box<List>), Nil }     // بدون Box: اندازهٔ بی‌نهایت!

// ۲) دادهٔ بزرگ: انتقال ارزان
let big = Box::new([0u64; 100_000]);        // فقط ۸ بایت اشاره‌گر جابه‌جا می‌شود
fn consume(b: Box<[u64; 100_000]>) { }

// ۳) trait object
let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { r: 1.0 })];

// ۴) تبدیل Vec → Box<[T]> (دو کلمه، سبک‌تر از Vec)
let slice: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
let text: Box<str> = "سلام".into();
```

---

## ۳) `Rc<T>` — مالکیت مشترک (یک‌تردی)

```rust
use std::rc::Rc;

let shared = Rc::new(vec![1, 2, 3]);
let a = Rc::clone(&shared);        // ← همیشه این سبک را بنویس، نه shared.clone()
let b = Rc::clone(&shared);
println!("{}", Rc::strong_count(&shared));   // 3
```
* 🎯 `Rc::clone(&x)` = فقط شمارنده +۱ (ارزان). `x.clone()` = خواننده فکر می‌کند کپی گره است.
* کمتر از `Rc` استفاده کن! گاهی شاخص (`TaskId`) یا `&T` تمیزتر است (همان کاری که
  در `app.rs` کردیم).
* ⚠️ `Rc` شمارندهٔ **غیراتمیک** دارد → `Send` نیست → بین تردها ممنوع (خوشبختانه
  کامپایلر جلویش را می‌گیرد).

---

## ۴) `RefCell<T>` — تغییرپذیری در زمان اجرا

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

let r1 = data.borrow();          // قرض ثابت (نگهبان)
let r2 = data.borrow();          // ✅ چند تای دیگر هم OK
println!("{} {}", r1.len(), r2.len());
drop(r1); drop(r2);

data.borrow_mut().push(4);       // ✅ بعد از آزادشدن قرض‌ها
```
| متد | می‌دهد | خطا |
|-----|--------|-----|
| `borrow()` | `Ref<T>` | 💥 پنیک اگر `borrow_mut` فعال باشد |
| `borrow_mut()` | `RefMut<T>` | 💥 پنیک اگر هر قرض دیگری فعال باشد |
| `try_borrow()` / `try_borrow_mut()` | `Result<…, BorrowError>` | ✅ بدون پنیک |

🎯 **همیشه `try_borrow*` بزن** مگر مطمئن باشی — چون قواعد قرض در زمان اجرا
چک می‌شوند و خطایشان «پنیک» است، نه خطای کامپایل.

### `Cell<T>` برای انواع `Copy`
```rust
use std::cell::Cell;
let hits = Cell::new(0);
let inc = || hits.set(hits.get() + 1);   // ← &self کافی است!
```
`Cell` هیچ قرضی نمی‌دهد (فقط get/set/replace/take) — پس نمی‌توانی مرجع بگیری،
ولی هم‌زمان نمی‌تواند پنیک کند.

---

## ۵) ترکیب `Rc<RefCell<T>>` — گراف قابل تغییر

```rust
struct Node {
    name: String,
    children: RefCell<Vec<Rc<Node>>>,      // ← قابل تغییر
}

impl Node {
    fn add_child(self: &Rc<Self>, child: Rc<Node>) {   // ← &Rc<Self> عمداً
        self.children.borrow_mut().push(child);
    }
}
```
این ترکیب در DOM، گراف صحنه (scene graph)، کش مشترک و … رایج است.
📌 هزینه‌اش: بررسی در زمان اجرا + احتمال پنیک. پس وقتی طراحی ساده‌تر ممکن است،
همان را انتخاب کن (اغلب شاخص‌ها کافی‌اند).

---

## ۶) چرخهٔ حافظه و `Weak`

```rust
struct Parent { child: RefCell<Option<Rc<Child>>> }
struct Child  { parent: RefCell<Weak<Parent>> }    // ← Weak، نه Rc!
```
اگر هر دو `Rc` باشند، شمارندهٔ هر دو هیچ‌وقت صفر نمی‌شود → **نشت حافظه** و
`Drop` هرگز اجرا نمی‌شود (دموی `smart_pointers` همین را چاپ می‌کند).

```rust
let weak = Rc::downgrade(&parent);       // اشارهٔ ضعیف
if let Some(p) = weak.upgrade() {        // ← Option: ممکن است مرده باشد
    println!("{}", p.name);
}
```
* `weak_count` با `Rc::weak_count` قابل خواندن است.
* 🎯 الگو: **جهت قوی = والد→فرزند، جهت ضعیف = فرزند→والد.**
* 🔑 هر `Rc` که از `upgrade()` می‌گیری، عمرش را خودت مدیریت کن (اگر آن را در
  جایی ذخیره کنی، شمارنده بالا می‌ماند).

---

## ۷) `Arc<Mutex<T>>` — اشتراک بین تردها

```rust
let counter = Arc::new(Mutex::new(0usize));

let mut handles = vec![];
for _ in 0..8 {
    let c = Arc::clone(&counter);
    handles.push(std::thread::spawn(move || {
        for _ in 0..1000 {
            *c.lock().unwrap() += 1;      // ← نگهبان در پایان این خط آزاد می‌شود
        }
    }));
}
for h in handles { h.join().unwrap(); }
```

### 📏 قواعد طلایی قفل
1. **قفل را کوتاه نگه دار:** `{ let mut g = m.lock()?; g.push(x); }` نه کل بدنهٔ تابع.
2. **قفل‌ها را همیشه با یک ترتیب ثابت بگیر** (جلوگیری از بن‌بست).
3. ⚠️ دام معروف: **`if let` + قفل** — نگهبان تا پایان بلوک `if let` زنده می‌ماند:
```rust
if let Some(x) = map.lock().unwrap().get(&k) {    // قفل تا آخر if-let قفل می‌ماند!
    other.lock().unwrap().insert(k, x);           // 💀 احتمال بن‌بست
}
// ✅ بهتر:
let value = { map.lock().unwrap().get(&k).cloned() };
if let Some(x) = value { other.lock().unwrap().insert(k, x); }
```
4. «مسمومیت» (poisoning): اگر تردی حین نگه‌داشتن قفل پنیک کند، قفل مسموم می‌شود.
   `lock()` یک `Result` می‌دهد → `unwrap_or_else(|e| e.into_inner())` می‌گذارد از داده بگذری.

### اتمیک‌ها به‌جای قفل
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
static CALLS: AtomicUsize = AtomicUsize::new(0);
CALLS.fetch_add(1, Ordering::Relaxed);
CALLS.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst);
```
| ترتیب | کِی |
|-------|-----|
| `Relaxed` | فقط شمارش (پرتکرارترین انتخاب درست) |
| `Acquire`/`Release` | الگوی قفل دستی؛ انتشار داده بین تردها |
| `SeqCst` | وقتی شک داری (گران‌ترین) |

---

## ۸) `Cow<'a, B>` — یا قرض، یا مالک

```rust
use std::borrow::Cow;

fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains("  ") {
        Cow::Owned(input.split_whitespace().collect::<Vec<_>>().join(" "))
    } else {
        Cow::Borrowed(input)          // ← صفر تخصیص در مسیر رایج
    }
}
```
کاربرد واقعی: توابعی که «معمولاً تغییری لازم نیست» ولی «گاهی لازم است».
`Cow<Path>`, `Cow<[u8]>` هم مفیدند.

---

## ۹) `Deref` — کِی و کِی نه

```rust
impl Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}
```
✅ درست: برای **اشاره‌گرهای هوشمند** (`Box`, `Rc`, `Arc`, `MutexGuard`, `String`→`str`).
❌ غلط: برای پوشش‌های منطقی (newtype) — چون متدهای نوع درونی را بی‌سروصدا
افشا می‌کند و «هدف نوع» را گم می‌کند.

`Deref coercion` یعنی: `&String` خودکار `&str` می‌شود، `&Vec<T>` خودکار `&[T]`.
همین باعث می‌شود `takes_str(&my_string)` بدون `as_str()` کار کند.

---

## ۱۰) ترفندهای ریز

```rust
// Box::leak — عمر 'static در ازای نشت حافظه (برای data ساختاری که تا آخر لازم است)
let s: &'static str = Box::leak(String::from("x").into_boxed_str());

// ManuallyDrop و Box::into_raw — برای FFI (به ۱۱ نگاه کن)

// Rc::try_unwrap — اگر تنها مالک هستی، داده را بیرون بکش (بدون کپی)
let rc = Rc::new(vec![1, 2, 3]);
let v = Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone());

// RefCell::replace / take
let cell = RefCell::new(vec![1]);
let old = cell.replace(vec![2]);

// OnceCell / OnceLock — مقداردهی یک‌باره در زمان اجرا
use std::sync::OnceLock;
static CONF: OnceLock<String> = OnceLock::new();
let v = CONF.get_or_init(|| std::fs::read_to_string("f").unwrap_or_default());

// LazyLock (Rust 1.80+) — جایگزین تنبل و راحت
static TABLE: std::sync::LazyLock<Vec<u32>> = std::sync::LazyLock::new(|| (1..=10).collect());
```

⚠️ **خودارجاعی (self-referential struct) در Rust ممنوع است** — ساختن ساختاری که
فیلدی به فیلد دیگرش اشاره کند، با `unsafe`/`Pin`/crate (`ouroboros`, `self_cell`) ممکن است.
راه ساده: دو ساختار جدا، یا شاخص، یا `Cow`.

[⬅️ قبلی: ایتراتور و بستار](06-iterators-and-closures.md) · [بعدی: هم‌روندی ➡️](08-concurrency.md)
