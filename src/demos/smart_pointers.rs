//! # دموی اشاره‌گرهای هوشمند
//!
//! | نوع | مالکیت | تغییرپذیری | اشتراک | کاربرد |
//! |-----|--------|-------------|--------|--------|
//! | `Box<T>` | یکتا | عادی | ❌ | بازگشت، اندازهٔ بزرگ، `dyn` |
//! | `Rc<T>` | چندگانه (ضریب‌دار) | ❌ | ✅ (یک‌تردی) | گراف/درخت مشترک |
//! | `Arc<T>` | چندگانه | ❌ | ✅ (چندتردی) | اشتراک بین تردها |
//! | `RefCell<T>` | یکتا | ✅ (در زمان اجرا) | ❌ | تغییرپذیری درونی |
//! | `Cell<T>` | یکتا | ✅ (کپی) | ❌ | فلگ‌های ساده |
//! | `Weak<T>` | غیرمالک | — | ✅ | شکستن چرخهٔ حافظه |


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) Box — ساده‌ترین اشاره‌گر هوشمند"));
    boxes();

    println!("\n{}", color::bold("۲) Rc — مالکیت مشترک با شمارش"));
    reference_counting();

    println!("\n{}", color::bold("۳) RefCell — تغییرپذیری در زمان اجرا"));
    interior_mutability();

    println!("\n{}", color::bold("۴) ترکیب Rc<RefCell<>> — گراف قابل تغییر"));
    graph();

    println!("\n{}", color::bold("۵) چرخهٔ حافظه و نجات با Weak"));
    cycles();

    println!("\n{}", color::bold("۶) Arc<Mutex<>> — اشتراک بین تردها"));
    threads_sharing();

    println!("\n{}", color::bold("۷) Cell — تغییرپذیری برای انواع Copy"));
    cells();
}

// ═══════════════════════════════════════════════════════════════════════════
/// فهرست پیوندی — بدون `Box` **قابل تعریف نیست** (اندازهٔ بی‌نهایت می‌شد).
#[derive(Debug)]
enum List {
    /// گره با مقدار و ادامه
    Cons(i32, Box<List>),
    /// پایان
    Nil,
}

fn boxes() {
    // نکته: `Box<List>` در enum اندازه‌اش معلوم است (یک اشاره‌گر)
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Cons(3, Box::new(List::Nil))))));
    println!("  فهرست پیوندی با Box: {list:?}");
    println!("  size_of::<List>() = {} بایت (اگر Box نبود، بی‌نهایت بود!)", std::mem::size_of::<List>());

    // ۱) Box برای دادهٔ بزرگ: انتقال به تابع ارزان می‌شود
    let big = Box::new([0u64; 1024]);
    println!("  جعبهٔ بزرگ: {} بایت روی heap، فقط ۸ بایت اشاره‌گر روی پشته", std::mem::size_of_val(&big));
    fn consume(b: Box<[u64; 1024]>) -> u64 {
        b.iter().sum()
    }
    println!("  جمع جعبه: {}", consume(big)); // انتقال ارزان (فقط اشاره‌گر جابه‌جا شد)

    // ۲) Box<dyn Trait> — رفتار پویا
    let shapes: Vec<Box<dyn std::fmt::Debug>> = vec![Box::new(1), Box::new("سلام"), Box::new(vec![1, 2])];
    println!("  Box<dyn Debug>: {shapes:?}");

    // ۳) Box<str> و Box<[T]> — نسخه‌های «ثابت» و سبک‌تر از String/Vec
    let boxed_str: Box<str> = "سلام".into();
    let boxed_slice: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    println!("  Box<str> = {boxed_str}، Box<[i32]> = {boxed_slice:?}");
    println!("  (Box<[T]> سه کلمه نیست؛ دو کلمه است → سبک‌تر از Vec)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// گره اشتراکی درخت — یک گره، چند مالک.
#[derive(Debug)]
struct SharedNode {
    /// نام گره
    name: String,
    /// فرزندان (اشتراکی)
    children: Vec<Rc<SharedNode>>,
}

fn reference_counting() {
    // برگ مشترک: هم زیر «الف» می‌آید هم زیر «ب» — یک نسخه در حافظه، دو مالک
    let leaf = Rc::new(SharedNode { name: "برگ".into(), children: vec![] });
    println!("  شمارش مالک‌ها بعد از ساخت: {}", Rc::strong_count(&leaf));

    let holder1 = SharedNode { name: "الف".into(), children: vec![Rc::clone(&leaf)] };
    println!("  بعد از افزودن به «الف»: {}", Rc::strong_count(&leaf));

    let holder2 = SharedNode { name: "ب".into(), children: vec![Rc::clone(&leaf)] };
    println!("  بعد از افزودن به «ب»:  {}", Rc::strong_count(&leaf));

    println!("  درخت: {holder1:?} و {holder2:?}");
    // 🔑 ترفند: همیشه `Rc::clone(&x)` بنویس (نه `x.clone()`)، تا خواننده بداند
    //    اینجا فقط شمارنده زیاد شده، نه کل داده کپی شده.
    println!("  Rc::clone فقط شمارنده را زیاد می‌کند — داده یک نسخه است.");

    drop(holder1);
    println!("  بعد از حذف «الف»: {}", Rc::strong_count(&leaf));
    drop(holder2);
    // leaf خودش هم یک مالک بود، پس هنوز ۱ است
    println!("  بعد از حذف «ب»:   {}", Rc::strong_count(&leaf));
    // ⚠️ یادآوری: Rc یک‌تردی است! برای بین تردها از Arc استفاده کن.
    println!("  (Rc شمارندهٔ غیراتمیک دارد → سبک‌تر ولی Single-Threaded)");
    println!("  size_of::<Rc<SharedNode>>() = {} بایت", std::mem::size_of::<Rc<SharedNode>>());
}

// ═══════════════════════════════════════════════════════════════════════════
/// شمارندهٔ مشترک با RefCell — «تغییرپذیری درونی».
#[derive(Debug, Default)]
struct Counter {
    value: RefCell<i32>,
    label: String,
}

impl Counter {
    fn increment(&self) {
        // توجه: `&self` نه `&mut self`! ولی مقدار تغییر می‌کند.
        *self.value.borrow_mut() += 1;
    }

    fn get(&self) -> i32 {
        *self.value.borrow()
    }
}

fn interior_mutability() {
    let c = Counter { value: RefCell::new(0), label: "تیک".into() };
    c.increment();
    c.increment();
    println!("  {} = {}", c.label, c.get());
    println!("  ({c:?})");

    // قواعد قرض در زمان اجرا:
    let cell = RefCell::new(vec![1, 2, 3]);

    // چند قرض ثابت هم‌زمان مجاز است (مثل حالت عادی)
    let b1 = cell.borrow();
    let b2 = cell.borrow();
    println!("  دو قرض ثابت هم‌زمان: {} و {}", b1.len(), b2.len());
    drop(b1);
    drop(b2);

    // ⚠️ قرض تغییرپذیر + ثابت هم‌زمان = پنیک در زمان اجرا!
    //     اینجا با `try_borrow_mut` امن چک می‌کنیم:
    let r = cell.borrow();
    match cell.try_borrow_mut() {
        Ok(_) => println!("  (غیرمنتظره) قرض تغییرپذیر موفق شد"),
        Err(e) => println!("  try_borrow_mut در حضور قرض ثابت → خطا (بدون پنیک): {e}"),
    }
    drop(r);

    // حالا که قرض تمام شد، تغییرپذیری آزاد است
    cell.borrow_mut().push(4);
    println!("  بعد از آزاد شدن: {cell:?}");
    println!("  💡 قاعدهٔ طلایی: همیشه `try_borrow*` بزن مگر مطمئن باشی خطر نیست");
}

// ═══════════════════════════════════════════════════════════════════════════
/// گره درخت با فرزند قابل اضافه‌شدن در زمان اجرا.
#[derive(Debug)]
struct TreeNode {
    name: String,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(name: &str) -> Rc<Self> {
        Rc::new(TreeNode { name: name.to_string(), children: RefCell::new(Vec::new()) })
    }

    /// افزودن فرزند — با `&self` (بدون `mut`) چون تغییرپذیری درونی داریم.
    fn add_child(self: &Rc<Self>, child: Rc<TreeNode>) {
        self.children.borrow_mut().push(child);
    }

    /// شمارش بازگشتی همهٔ گره‌ها.
    fn count(&self) -> usize {
        1 + self.children.borrow().iter().map(|c| c.count()).sum::<usize>()
    }

    /// چاپ درختی.
    fn print(&self, depth: usize) {
        println!("    {}{}", "─ ".repeat(depth), self.name);
        for c in self.children.borrow().iter() {
            c.print(depth + 1);
        }
    }
}

fn graph() {
    let root = TreeNode::new("ریشه");
    let a = TreeNode::new("الف");
    let b = TreeNode::new("ب");
    let leaf = TreeNode::new("برگ مشترک");

    root.add_child(Rc::clone(&a));
    root.add_child(Rc::clone(&b));
    a.add_child(Rc::clone(&leaf));
    b.add_child(Rc::clone(&leaf));

    root.print(0);
    println!("  تعداد گره‌ها = {} (برگ مشترک یک بار شمرده می‌شود)", root.count());
    println!("  strong_count(leaf) = {}", Rc::strong_count(&leaf));
    println!("  (با Rc<RefCell<>> می‌توانیم درخت را در زمان اجرا بسازیم و تغییر دهیم)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// گره والد ↔ فرزند: چرخهٔ حافظه.
#[derive(Debug)]
struct Parent {
    name: String,
    /// فرزند با مالکیت مشترک (قوی)
    child: RefCell<Option<Rc<Child>>>,
}

/// فرزند — به والد با `Weak` اشاره می‌کند (بدون مالکیت).
#[derive(Debug)]
struct Child {
    name: String,
    /// اشارهٔ ضعیف به والد → چرخه نمی‌سازد
    parent: RefCell<Weak<Parent>>,
}

fn cycles() {
    // ── الف) چرخهٔ واقعی: هر دو طرف `Rc` دارند → شمارنده‌ها هرگز صفر نمی‌شوند ──
    #[derive(Debug)]
    struct CyclicParent {
        name: String,
        child: RefCell<Option<Rc<CyclicChild>>>,
    }
    #[derive(Debug)]
    struct CyclicChild {
        name: String,
        parent: RefCell<Option<Rc<CyclicParent>>>,
    }

    // پیاده‌سازی Drop تا ببینیم «رها شدن» واقعاً رخ می‌دهد یا نه
    impl Drop for CyclicParent {
        fn drop(&mut self) {
            println!("    ↓ Drop والد اجرا شد ({})", self.name);
        }
    }
    impl Drop for CyclicChild {
        fn drop(&mut self) {
            println!("    ↓ Drop فرزند اجرا شد ({})", self.name);
        }
    }

    println!("  الف) با Rc در هر دو جهت (اشتباه):");
    {
        let p = Rc::new(CyclicParent { name: "والد".into(), child: RefCell::new(None) });
        let c = Rc::new(CyclicChild { name: "فرزند".into(), parent: RefCell::new(None) });
        *p.child.borrow_mut() = Some(Rc::clone(&c)); // والد → فرزند (قوی)
        *c.parent.borrow_mut() = Some(Rc::clone(&p)); // فرزند → والد (قوی) ← 💥 چرخه!
        println!("    strong(parent)={} strong(child)={}", Rc::strong_count(&p), Rc::strong_count(&c));
    } // اینجا هر دو متغیر بیرونی رها می‌شوند…
    println!("    ⚠️ توجه: هیچ‌کدام از دو «Drop» بالا چاپ نشد → حافظه نشت کرد!
");

    // ── ب) راه درست: یک جهت با Weak ──
    println!("  ب) با Weak در جهت برگشت (درست):");
    let parent: Rc<Parent> = Rc::new(Parent { name: "والد".into(), child: RefCell::new(None) });
    let child: Rc<Child> = Rc::new(Child { name: "فرزند".into(), parent: RefCell::new(Rc::downgrade(&parent)) });

    *parent.child.borrow_mut() = Some(Rc::clone(&child));
    println!("    strong(parent)={} strong(child)={} weak(parent)={}",
        Rc::strong_count(&parent), Rc::strong_count(&child), Rc::weak_count(&parent));

    // از طرف فرزند، والد را پیدا کن (با upgrade؛ ممکن است مرده باشد)
    let back = child.parent.borrow().upgrade();
    println!("    فرزند ← والد: {:?}", back.as_ref().map(|p| p.name.as_str()));
    drop(back); // 🔑 هر Rc حاصل از upgrade را رها کن، وگرنه شمارش بالا می‌ماند

    // حالا والد را حذف کن → اشارهٔ ضعیف امن به None تبدیل می‌شود (بدون پنیک)
    drop(parent);
    println!("    بعد از حذف والد: upgrade → {:?}", child.parent.borrow().upgrade().map(|p| p.name.clone()));
    println!("    ✅ Weak نه چرخه می‌سازد و نه اشارهٔ آویزان (dangling) می‌دهد");
    println!("  📌 قاعده: در گراف، «جهت قوی» را والد→فرزند و «جهت ضعیف» را فرزند→والد بگذار");
}

// ═══════════════════════════════════════════════════════════════════════════
fn threads_sharing() {
    // Arc = Atomic Reference Counted → شمارندهٔ اتمیک، امن بین تردها
    let shared = Arc::new(Mutex::new(Vec::<String>::new()));

    let mut handles = Vec::new();
    for i in 0..4 {
        // `Arc::clone` → شمارنده بالا می‌رود؛ داده یک نسخه است
        let shared = Arc::clone(&shared);
        handles.push(std::thread::spawn(move || {
            // `lock()` قفل را می‌گیرد؛ وقتی نگهبان از دامنه بیرون رفت آزاد می‌شود.
            // 🔸 ترفند: `unwrap_or_else(|e| e.into_inner())` = «از قفل مسموم نترس»
            let mut guard = shared.lock().unwrap_or_else(|e| e.into_inner());
            guard.push(format!("پیام از ترد {i}"));
        }));
    }

    for h in handles {
        h.join().expect("ترد پنیک کرد");
    }

    let mut all = shared.lock().unwrap_or_else(|e| e.into_inner());
    all.sort();
    println!("  محتوای مشترک: {all:?}");
    drop(all);

    // ترفند: مسیر بهینه‌سازی — «قفل را کوتاه نگه دار»
    // هرچه بین lock و drop کار کمتری کنی، رقابت کم‌تر و برنامه سریع‌تر است.
    // (یک تلهٔ معروف: نگه‌داشتن قفل روی `&` در `if let` — به docs/14 نگاه کن)

    // ترفند: اگر فقط خواندن لازم است، RwLock اجازه می‌دهد چند خوانندهٔ هم‌زمان داشته باشی
    let rw = std::sync::RwLock::new(10);
    {
        let r1 = rw.read().unwrap();
        let r2 = rw.read().unwrap();
        println!("  RwLock: دو خوانندهٔ هم‌زمان → {} و {}", *r1, *r2);
    }
    *rw.write().unwrap() += 5;
    println!("  بعد از نوشتن: {}", *rw.read().unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
fn cells() {
    // Cell<T> برای انواع Copy: هیچ قرضی نمی‌دهد، فقط «کپی در/کپی بیرون»
    let hits = Cell::new(0);

    let count_visit = || {
        hits.set(hits.get() + 1); // &self کافی است! چون فقط کپی می‌شود
    };
    count_visit();
    count_visit();
    count_visit();
    println!("  Cell: بازدیدها = {}", hits.get());
    println!("  size_of::<Cell<i32>>() = {} (هم‌اندازهٔ i32 — هزینهٔ صفر)", std::mem::size_of::<Cell<i32>>());

    // ترفند معروف: با Cell می‌توان «شمارندهٔ درون بستار» ساخت بدون RefCell
    let call_count = Cell::new(0usize);
    let f = |x: i32| {
        call_count.set(call_count.get() + 1);
        x * x
    };
    let _: Vec<i32> = (1..=5).map(f).collect();
    println!("  شمارش فراخوانی با Cell: {}", call_count.get());

    // Cell::take / Cell::replace — بدون هیچ قرض‌گیری
    let name = Cell::new(String::from("الف"));
    let old = name.replace(String::from("ب"));
    println!("  Cell::replace → قبلی={old:?} فعلی={:?}", name.take());

    println!("  📌 انتخاب: مقدار Copy → Cell | دادهٔ پیچیده → RefCell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn rc_counts_are_accurate() {
        let leaf = Rc::new(SharedNode { name: "x".into(), children: vec![] });
        assert_eq!(Rc::strong_count(&leaf), 1);
        let _c = Rc::clone(&leaf);
        assert_eq!(Rc::strong_count(&leaf), 2);
    }

    #[test]
    fn refcell_try_borrow_does_not_panic() {
        let c = RefCell::new(5);
        let _r = c.borrow();
        assert!(c.try_borrow_mut().is_err());
        drop(_r);
        assert!(c.try_borrow_mut().is_ok());
    }

    #[test]
    fn weak_upgrade_becomes_none_after_drop() {
        let parent = Rc::new(Parent { name: "p".into(), child: RefCell::new(None) });
        let weak: Weak<Parent> = Rc::downgrade(&parent);
        assert!(weak.upgrade().is_some());
        drop(parent);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn tree_count_traverses_shared_nodes() {
        let root = TreeNode::new("r");
        let leaf = TreeNode::new("leaf");
        root.add_child(Rc::clone(&leaf));
        root.add_child(Rc::clone(&leaf));
        assert_eq!(root.count(), 3); // ریشه + دو ارجاع به برگ
    }

    #[test]
    fn arc_mutex_accumulates_across_threads() {
        let data = Arc::new(Mutex::new(0usize));
        std::thread::scope(|s| {
            for _ in 0..4 {
                let d = Arc::clone(&data);
                s.spawn(move || {
                    for _ in 0..250 {
                        *d.lock().unwrap() += 1;
                    }
                });
            }
        });
        assert_eq!(*data.lock().unwrap(), 1000);
    }
}
