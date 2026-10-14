# 🔄 ۰۶ — ایتراتورها و بستارها

## چرا ایتراتور؟

```rust
// سبک حلقه‌ای (indexed)
let mut sum = 0;
for i in 0..v.len() { sum += v[i]; }

// سبک ایتراتوری (idiomatic Rust)
let sum: i32 = v.iter().sum();
```
مزایا: بدون خطر ایندکس، قابل ترکیب، **همان‌قدر سریع یا سریع‌تر** (کامپایلر
حلقه را درون‌ریزی می‌کند و مرزها را حذف می‌کند).

---

## ۱) تنبلی (laziness)

```rust
let iter = v.iter().map(|x| { println!("اجرا شدم!"); x * 2 });  // هیچ چاپی نمی‌بینی
let out: Vec<_> = iter.collect();                                // ← حالا اجرا می‌شود
```
* **آداپتورها** (تنبل): `map`, `filter`, `filter_map`, `flat_map`, `take`, `skip`,
  `take_while`, `skip_while`, `chain`, `zip`, `enumerate`, `peekable`, `rev`,
  `step_by`, `cloned`, `copied`, `scan`, `inspect`, `fuse`, `by_ref`, `cycle`
* **مصرف‌کننده‌ها** (اجراکننده): `collect`, `fold`, `reduce`, `sum`, `product`,
  `count`, `any`, `all`, `find`, `find_map`, `position`, `min`, `max`, `min_by_key`,
  `last`, `nth`, `partition`, `unzip`, `for_each`, `try_fold`, `try_for_each`

🎯 مزیت واقعی تنبلی: روی ایتراتور **بی‌نهایت** می‌توان کار کرد:
```rust
let first_three: Vec<u64> = (1..).filter(|x| x % 7 == 0).take(3).collect();
```

---

## ۲) دستور پخت‌های پرکاربرد

```rust
// ۱) گروه‌بندی
let mut groups: HashMap<char, Vec<&str>> = HashMap::new();
for w in words { groups.entry(w.chars().next().unwrap_or('?')).or_default().push(w); }

// ۲) شمارش
for w in words { *counts.entry(w).or_insert(0) += 1; }

// ۳) یکتاسازی با حفظ ترتیب
let mut seen = HashSet::new();
let unique: Vec<_> = items.iter().filter(|x| seen.insert(*x)).collect();

// ۴) تخت‌کردن
let flat: Vec<_> = matrix.iter().flat_map(|row| row.iter()).collect();

// ۵) پنجرهٔ لغزان
let avg: Vec<f64> = prices.windows(3).map(|w| w.iter().sum::<f64>() / 3.0).collect();

// ۶) جمع تجمعی
let cumulative: Vec<i32> = data.iter().scan(0, |acc, x| { *acc += x; Some(*acc) }).collect();

// ۷) زنجیرهٔ حالت (Collatz، فاکتوریل، …)
let seq: Vec<u64> = std::iter::successors(Some(27), |&n| (n != 1).then(|| if n % 2 == 0 { n / 2 } else { 3 * n + 1 })).take(10).collect();

// ۸) N مورد بزرگ‌تر بدون مرتب‌سازی کل
let mut heap = BinaryHeap::new();
for x in data { heap.push(Reverse(x)); if heap.len() > 3 { heap.pop(); } }

// ۹) تمام‌شدن روی اولین خطا
let ok: Result<Vec<_>, _> = items.iter().map(parse).collect();

// ۱۰) همهٔ خطاها را جدا کن
let (good, bad): (Vec<_>, Vec<_>) = items.iter().map(parse).partition(Result::is_ok);
```

---

## ۳) ایتراتور دلخواه

```rust
struct Counter { count: u32 }

impl Iterator for Counter {
    type Item = u32;                       // ← «نوع وابسته»

    fn next(&mut self) -> Option<Self::Item> {
        (self.count < 5).then(|| { self.count += 1; self.count })
    }

    // ⚡ بهینه‌سازی: بگو چند تا مانده، تا `collect` از قبل حافظه بگیرد
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (5 - self.count) as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for Counter {}      // چون size_hint دقیق است
impl std::iter::FusedIterator for Counter {} // تضمین: بعد از None همیشه None
```

### صفت‌های مکمل
| صفت | چه می‌دهد |
|-----|-----------|
| `ExactSizeIterator` | `.len()` |
| `DoubleEndedIterator` | `.rev()` |
| `FusedIterator` | بهینه‌سازی: بعد از `None` دوباره صدا نزن |
| `Clone` | `.clone()` روی ایتراتور (برای پیمایش دوباره) |
| `Send`/`Sync` | انتقال بین تردها |

🎯 **بهترین راه:** اگر ایتراتور دلخواه فقط برای یک تابع است، از
`std::iter::from_fn` یا `successors` استفاده کن — بدون نوشتن struct:

```rust
let mut n = 0;
let squares = std::iter::from_fn(|| { n += 1; (n <= 5).then(|| n * n) });
```

---

## ۴) بستارها (closures)

```rust
let add = |a: i32, b: i32| a + b;      // با نوع
let double = |x| x * 2;                // استنتاج از کاربرد
let nothing = || {};                   // بدون آرگومان
let block = |x| { let y = x + 1; y * 2 };   // بدنهٔ چندخطی
```

### سه صفت فراخوانی
| صفت | محیط را | چند بار | توضیح |
|-----|---------|---------|-------|
| `Fn` | قرض ثابت | چند بار | فقط می‌خواند |
| `FnMut` | قرض تغییرپذیر | چند بار | تغییر می‌دهد (مثل شمارنده) |
| `FnOnce` | مصرف | یک بار | مقدار را برمی‌گرداند/jابه‌جا می‌کند |

> هر `Fn` یک `FnMut` است، و هر `FnMut` یک `FnOnce`.
> **پس همیشه بی‌سخت‌ترین را بپذیر:** اگر `Fn` کافی است، `FnOnce` نپذیر.

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }         // فقط می‌خواند
fn run_twice<F: FnMut()>(mut f: F) { f(); f(); }                  // تغییر می‌دهد
fn consume<F: FnOnce() -> String>(f: F) -> String { f() }         // یک بار
fn take_box(f: Box<dyn FnOnce() + Send + 'static>) { /* در ترد */ }
```

### گرفتن محیط: سه حالت
```rust
let s = String::from("x");

let by_ref = || println!("{s}");          // قرض (s بعدش قابل استفاده است)
let mut v = vec![];
let mut by_mut = || v.push(1);            // قرض تغییرپذیر
let by_move = move || s;                  // انتقال مالکیت (s دیگر مال ما نیست)
```

### برگرداندن بستار از تابع
```rust
fn adder(n: i32) -> impl Fn(i32) -> i32 { move |x| x + n }
//                                    ^^^^ بدون move → خطای عمر (محیط محلی می‌میرد)

fn maker(n: i32) -> Box<dyn Fn(i32) -> i32> { Box::new(move |x| x + n) }  // انواع مختلف
```

### بستارها هرگز `Copy`/`Clone` نیستند مگر محیطشان باشد
```rust
let x = 5;                                 // i32: Copy
let f = move || x;                         // بستار Copy است
let g = f;                                 // کپی شد! f هم قابل استفاده است
let g2 = f();
```

### محدودیت‌ها
* ⚠️ هر بستار فقط با **یک نوع** پارامتر کار می‌کند (برخلاف تابع ژنریک).
* ⚠️ `let id = |x| x; id(1); id("s");` خطا می‌دهد.
* ⚠️ اگر بستار را در ساختار ذخیره می‌کنی و عمر دارد، `Box<dyn Fn(...) + 'a>` یا پارامتر نوع لازم است.

---

## ۵) `Iterator` + بستار = ترکیب برنده

```rust
let people = vec![("علی", 30), ("زهرا", 25)];

people.sort_by(|a, b| a.1.cmp(&b.1));          // ⚠️ sort_by_key با مرجع کار نمی‌کند
people.sort_by_key(|p| p.1);                    // ✅ روی Copy

let names: Vec<&str> = people.iter().map(|(n, _)| *n).collect();
let older: Vec<_> = people.iter().filter(|(_, age)| **age > 26).collect();
let total: i32 = people.iter().map(|(_, a)| a).sum();

// الگوی «تجزیه در پارامتر بستار»: |(n, _)| — خیلی خوانا!
```

### حافظه‌سازی (memoization) با `RefCell`
```rust
let cache: RefCell<HashMap<u64, u64>> = RefCell::new(HashMap::new());
let fib = |n: u64| -> u64 {
    if let Some(v) = cache.borrow().get(&n) { return *v; }
    let v = slow_fib(n);
    cache.borrow_mut().insert(n, v);
    v
};
```
🎯 اینجا بستار هنوز `Fn` است (نه `FnMut`) چون `RefCell` تغییرپذیری درونی دارد.

---

## ۶) کارایی — افسانه‌ها و واقعیت

| افسانه | واقعیت |
|--------|--------|
| «حلقه سریع‌تر از ایتراتور است» | ❌ معمولاً برعکس: ایتراتور مرزها را حذف می‌کند |
| «`collect` گران است» | فقط اگر لازم باشد؛ و با `size_hint` از قبل تخصیص می‌گیرد |
| «`.clone()` بی‌خطر است» | ❌ هر clone یک کپی روی heap است — پروفایل کن |
| «زنجیرهٔ طولانی کند است» | ❌ کامپایلر آن را به یک حلقه تبدیل می‌کند |

مثال واقعی از این پروژه:
```rust
// در app.rs — بدون هیچ تخصیص اضافی
let rows: Vec<&Task> = self.tasks.iter().filter(|t| filter.accepts(t)).collect();
```

🎯 برای پیمایش بدون تخصیص، به‌جای `collect` خروجی `impl Iterator` بده:
```rust
pub fn pending(&self) -> impl Iterator<Item = &Task> {
    self.tasks.iter().filter(|t| !t.done)
}
```

### نکاتی که سرعت را واقعاً عوض می‌کنند
1. `Vec::with_capacity(n)` قبل از حلقهٔ push.
2. `String::with_capacity(n)` قبل از کنار هم چسباندن.
3. `&str` پارامتر بگیر، نه `String`.
4. `retain` به‌جای build مجدد.
5. `entry` API به‌جای `get` + `insert` (دو بار هش).
6. `iter()` روی برش به‌جای `.get(i)` در حلقه.
7. `binary_search` روی دادهٔ مرتب.
8. `#[inline]` روی توابع کوچک hot (کامپایلر معمولاً خودش می‌فهمد).

---

## ۷) الگوی «زنجیره‌ای کردن» در کد واقعی

از همین پروژه (`src/app.rs`، تابع `stats`):

```rust
let done = self.tasks.iter().filter(|t| t.done).count();

let durations: Vec<u64> = self.tasks.iter().filter_map(Task::time_to_done).collect();

let oldest_pending = self.tasks.iter()
    .filter(|t| !t.done)
    .min_by_key(|t| t.created_at)
    .map(|t| t.id);

let urgency: u32 = self.tasks.iter()
    .filter(|t| !t.done)
    .map(|t| t.priority.weight())
    .sum();
```
📌 تکنیکی که باید یاد بگیری: **`filter_map`** = فیلتر + تبدیل در یک گام (جاهایی که
`Option` داری). و **`min_by_key`/`max_by_key`** که کلید مقایسه می‌سازند.

[⬅️ قبلی: خطاها](05-errors.md) · [بعدی: اشاره‌گرهای هوشمند ➡️](07-smart-pointers.md)
