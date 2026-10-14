//! # دموی مجموعه‌ها (Collections) — انتخاب ظرف درست
//!
//! راهنمای سریع انتخاب:
//!
//! | نیاز | نوع |
//! |------|-----|
//! | فهرست ترتیبی، ایندکس‌دار | `Vec<T>` |
//! | صف دو-سر (push/pop از هر دو طرف) | `VecDeque<T>` |
//! | نگاشت کلید→مقدار بدون ترتیب (سریع‌ترین) | `HashMap<K, V>` |
//! | نگاشت **مرتب** یا جست‌وجوی محدوده‌ای | `BTreeMap<K, V>` |
//! | مجموعهٔ یگانه | `HashSet<T>` / `BTreeSet<T>` |
//! | بالاترین/پایین‌ترین اولویت | `BinaryHeap<T>` |


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) Vec — ستون فقرات برنامه‌های Rust"));
    vectors();

    println!("\n{}", color::bold("۲) HashMap و «API ورودی» (entry)"));
    hash_maps();

    println!("\n{}", color::bold("۳) BTreeMap — وقتی ترتیب مهم است"));
    btree();

    println!("\n{}", color::bold("۴) HashSet / BTreeSet — عملیات مجموعه‌ای"));
    sets();

    println!("\n{}", color::bold("۵) VecDeque — صف"));
    queues();

    println!("\n{}", color::bold("۶) BinaryHeap — صف اولویت‌دار"));
    heaps();
}

// ═══════════════════════════════════════════════════════════════════════════
fn vectors() {
    let mut v: Vec<i32> = Vec::new();
    v.push(3);
    v.push(1);
    v.extend([4, 1, 5]); // افزودن چندتا با هم
    println!("  v = {v:?}  (len={}, capacity={})", v.len(), v.capacity());

    // ترفند: `vec!` ماکرو برای ساخت سریع
    let mut nums = vec![5, 3, 1, 3, 5];
    nums.sort(); // مرتب‌سازی صعودی
    nums.dedup(); // حذف تکراری‌های *مجاور* (پس اول sort، بعد dedup!)
    println!("  sort+dedup = {nums:?}");

    // جست‌وجوی دودویی روی دادهٔ مرتب — O(log n)
    println!("  binary_search(3) = {:?}", nums.binary_search(&3));
    println!("  binary_search(2) = {:?}  ← برنگشتند؛ جایش را می‌دهد", nums.binary_search(&2));

    // دسترسی امن
    println!("  get(0) = {:?}، get(99) = {:?}  ← بدون پنیک", nums.get(0), nums.get(99));
    println!("  first()/last() = {:?}/{:?}", nums.first(), nums.last());
    // `v[99]` پنیک می‌کند — همان دام؛ در کد تولیدی از `get` استفاده کن.

    // پنجره‌ها و تکه‌ها (برای پردازش سیگنال/داده)
    let data = [1, 2, 3, 4, 5];
    println!("  windows(2): {:?}", data.windows(2).map(|w| w.iter().sum::<i32>()).collect::<Vec<_>>());
    println!("  chunks(2): {:?}", data.chunks(2).collect::<Vec<_>>());

    // برش و تقسیم
    let mut split = vec![1, 2, 3, 4, 5, 6];
    let tail = split.split_off(3); // نصفهٔ دوم را جدا می‌کند
    println!("  split_off(3) → اول={split:?} دوم={tail:?}");

    // `drain` = گرفتن + حذف
    let drained: Vec<i32> = split.drain(..2).collect();
    println!("  drain(..2) → کشیده‌شده={drained:?} باقی‌مانده={split:?}");

    // درج و حذف در وسط (کند: O(n) — زیرا باید بقیه جابه‌جا شوند)
    split.insert(0, 99);
    let removed = split.remove(0);
    println!("  insert(0,99) و remove(0)={removed} → {split:?}");

    // ترفند کارایی: `swap_remove` به‌جای `remove` وقتی ترتیب مهم نیست (O(1))
    let mut s = vec!['a', 'b', 'c'];
    let x = s.swap_remove(0);
    println!("  swap_remove(0)={x} → {s:?}  ← آخرین عنصر جای خالی را پر می‌کند");

    // نگه‌داری فقط آنچه می‌خواهیم
    let mut scores = vec![10, -5, 30, -1, 20];
    scores.retain(|s| *s > 0);
    println!("  retain(>0) → {scores:?}");

    // ظرفیت: جلوگیری از تخصیص مکرر
    let mut pre = Vec::with_capacity(1000);
    for i in 0..1000 {
        pre.push(i);
    }
    println!("  با with_capacity(1000): len={} capacity={} (بدون تخصیص اضافی)", pre.len(), pre.capacity());
}

// ═══════════════════════════════════════════════════════════════════════════
fn hash_maps() {
    let mut counts: HashMap<String, usize> = HashMap::new();

    // ۱) شمارش معمولی
    for word in ["rust", "go", "rust", "rust", "go"] {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    println!("  شمارش: {counts:?}");

    // ۲) 🌟 «API ورودی» — قلاب‌های مختلفش:
    let mut m: HashMap<&str, Vec<i32>> = HashMap::new();
    m.entry("زوج").or_default().push(2); // or_default = or_insert_with(Vec::new)
    m.entry("زوج").or_default().push(4);
    m.entry("فرد").or_default().push(1);
    println!("  گروه‌بندی: {m:?}");

    let mut hits: HashMap<&str, i32> = HashMap::new();
    hits.entry("a").and_modify(|v| *v += 100).or_insert(7);
    hits.entry("a").and_modify(|v| *v += 100).or_insert(7);
    println!("  and_modify: {hits:?}  (بار اول ۷، بار دوم ۱۰۷، بار سوم ۲۰۷)");

    // ۳) شمارش بدون `entry` (سبک کلاسیک) — برای مقایسه
    let mut m2 = HashMap::new();
    for k in ["x", "y", "x"] {
        let c = m2.get(k).copied().unwrap_or(0);
        m2.insert(k, c + 1);
    }
    println!("  دستی: {m2:?}");

    // ترفند: `get` مقدار مرجع می‌دهد؛ `.copied()` برای انواع Copy راحت‌ترین راه است.
    println!("  get(\"rust\") = {:?}", counts.get("rust").copied());

    // ترفند: `entry(key).or_insert_with(|| expensive())` — محاسبهٔ سنگین فقط وقتی لازم است
    let mut cache: HashMap<u32, u64> = HashMap::new();
    let v = *cache.entry(12).or_insert_with(|| {
        println!("    (فقط یک بار محاسبه می‌شود)");
        12u64 * 12
    });
    println!("  or_insert_with → {v}");

    // ترفند: HashMap با کلید tuple — عملاً یک ماتریس تُنُک
    let mut sparse: HashMap<(u32, u32), f64> = HashMap::new();
    sparse.insert((1, 2), 3.5);
    sparse.insert((1000, 1000), 9.9);
    println!("  ماتریس تُنُک با {} عنصر (حافظه فقط برای همان دو خانه)", sparse.len());

    // ترفند: تفاوت HashMap و BTreeMap را با نمونهٔ واقعی ببین
    // HashMap: بدون ترتیب (چون هش تصادفی است) → خروجی غیرقابل پیش‌بینی
    // (این «تصادفی بودن» ویژگی امنیتی است: جلوی حملهٔ DoS با کلیدهای تصادفی را می‌گیرد)
}

// ═══════════════════════════════════════════════════════════════════════════
fn btree() {
    let mut scores: BTreeMap<&str, i32> = BTreeMap::new();
    for (name, score) in [("علی", 90), ("زهرا", 95), ("رضا", 80), ("مریم", 85)] {
        scores.insert(name, score);
    }

    println!("  پیمایش (همیشه مرتب بر اساس کلید):");
    for (k, v) in &scores {
        println!("    {k} = {v}");
    }

    // 🌟 مزیت اصلی BTreeMap: پرس‌وجوی محدوده‌ای (range)
    let from = scores.range("ر"..).map(|(k, _)| *k).collect::<Vec<_>>();
    println!("  کلیدهای از «ر» به بعد: {from:?}");

    // اولین/آخرین، بزرگ‌تر از X
    println!("  first_key_value = {:?}", scores.first_key_value());
    println!("  last_key_value  = {:?}", scores.last_key_value());
    println!("  range(..\"ر\").next = {:?}", scores.range(.."ر").next());

    // نسخهٔ مجموعه‌ای BTreeSet هم مرتب است:
    let sorted: BTreeSet<i32> = [5, 1, 9, 1, 3].into_iter().collect();
    println!("  BTreeSet خودکار مرتب و بی‌تکرار: {sorted:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn sets() {
    let a: HashSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let b: HashSet<i32> = [3, 4, 5, 6].into_iter().collect();

    // توجه: این‌ها ایتراتور برمی‌گردانند (تنبل) پس باید collect کنی
    let inter: Vec<i32> = a.intersection(&b).copied().collect();
    let uni: Vec<i32> = a.union(&b).copied().collect();
    let diff: Vec<i32> = a.difference(&b).copied().collect();
    let sym: Vec<i32> = a.symmetric_difference(&b).copied().collect();

    let sort = |v: Vec<i32>| { let mut v = v; v.sort(); v };
    println!("  ∩ (اشتراک)     = {:?}", sort(inter));
    println!("  ∪ (اجتماع)     = {:?}", sort(uni));
    println!("  ∖ (تفاضل)      = {:?}", sort(diff));
    println!("  △ (تفاضل متقارن) = {:?}", sort(sym));

    println!("  is_subset/is_disjoint: {}/{}", a.is_subset(&b), a.is_disjoint(&b));

    // کاربرد واقعی: حذف تکراری‌ها با حفظ ترتیب
    let words = ["b", "a", "b", "c", "a"];
    let mut seen = HashSet::new();
    let unique: Vec<&str> = words.iter().copied().filter(|w| seen.insert(*w)).collect();
    println!("  یکتاسازی با حفظ ترتیب: {unique:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
fn queues() {
    let mut q: VecDeque<i32> = VecDeque::new();
    q.push_back(1);
    q.push_back(2);
    q.push_front(0); // O(1) — چیزی که Vec نمی‌تواند!
    println!("  صف: {q:?}");

    println!("  pop_front = {:?}، pop_back = {:?}", q.pop_front(), q.pop_back());
    println!("  بعد از pop: {q:?}");

    // حلقه با ظرفیت محدود (کاربرد: بافر لاگ)
    let mut log: VecDeque<&str> = VecDeque::with_capacity(3);
    for line in ["خط ۱", "خط ۲", "خط ۳", "خط ۴", "خط ۵"] {
        if log.len() == 3 {
            log.pop_front(); // قدیمی‌ترین را بیرون بینداز
        }
        log.push_back(line);
    }
    println!("  سه خط آخر (بافر حلقوی): {:?}", log);

    // ساخت از ایتراتور + چرخش
    let mut ring: VecDeque<i32> = (1..=5).collect();
    ring.rotate_left(2);
    println!("  rotate_left(2): {ring:?}");
    ring.rotate_right(1);
    println!("  rotate_right(1): {ring:?}");
    println!("  make_contiguous (تبدیل به برش پیوسته): {:?}", ring.make_contiguous());
}

// ═══════════════════════════════════════════════════════════════════════════
/// یک آیتم صف اولویت با ترتیب دلخواه.
#[derive(Debug, PartialEq, Eq)]
struct Job {
    /// اولویت بالاتر = زودتر
    priority: u8,
    /// نام کار
    name: String,
}

// برای BinaryHeap باید Ord پیاده شود. نکتهٔ کلیدی: ترتیب را *برعکس* می‌کنیم
// تا بزرگ‌ترین اولویت بالای heap بیاید. (راه دیگر: Wrapping با Reverse)
impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.name.cmp(&self.name)) // tie-break پایدار
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn heaps() {
    let mut jobs = BinaryHeap::new();
    jobs.push(Job { priority: 1, name: "نوشتن ایمیل".into() });
    jobs.push(Job { priority: 9, name: "رفع باگ بحرانی".into() });
    jobs.push(Job { priority: 5, name: "بازبینی کد".into() });

    println!("  ترتیب پردازش (بالاترین اولویت اول):");
    // `into_sorted_vec` ترتیب خروجی را از کم به زیاد می‌دهد؛ اینجا pop می‌کنیم تا طبیعی باشد
    while let Some(job) = jobs.pop() {
        println!("    [{}] {}", job.priority, job.name);
    }

    // ترفند پرکاربرد: با `Reverse` روی BinaryHeap، «کوچک‌ترین» مثل max-heap عمل می‌کند
    let mut smallest = BinaryHeap::new();
    for n in [5, 1, 9, 3] {
        smallest.push(std::cmp::Reverse(n));
    }
    println!("  Reverse → کوچک‌ترین‌ها اول: {:?}", smallest.into_sorted_vec());

    // ترفند: N عنصر بزرگ‌تر بدون مرتب‌سازی کل داده
    let data = [42, 7, 19, 88, 3, 55];
    let mut top = BinaryHeap::new();
    for n in data {
        top.push(std::cmp::Reverse(n));
        if top.len() > 3 {
            top.pop(); // کوچک‌ترین را بیرون کن → فقط ۳ بزرگ باقی می‌ماند
        }
    }
    let mut top3: Vec<i32> = top.into_iter().map(|r| r.0).collect();
    top3.sort_unstable_by(|a, b| b.cmp(a));
    println!("  سه مقدار بزرگ: {top3:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn entry_api_counts() {
        let mut m = HashMap::new();
        for w in ["a", "b", "a"] {
            *m.entry(w).or_insert(0) += 1;
        }
        assert_eq!(m["a"], 2);
        assert_eq!(m["b"], 1);
    }

    #[test]
    fn dedup_needs_sort_first() {
        let mut v = vec![1, 2, 1];
        v.dedup();
        assert_eq!(v, vec![1, 2, 1]); // چون مجاور نبودند!
        v.sort();
        v.dedup();
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn binary_heap_orders_by_priority() {
        let mut h = BinaryHeap::new();
        h.push(Job { priority: 1, name: "کم".into() });
        h.push(Job { priority: 9, name: "زیاد".into() });
        assert_eq!(h.pop().unwrap().priority, 9);
    }

    #[test]
    fn deque_ring_buffer_keeps_last_n() {
        let mut ring: VecDeque<i32> = VecDeque::new();
        for i in 1..=5 {
            if ring.len() == 3 { ring.pop_front(); }
            ring.push_back(i);
        }
        assert_eq!(ring, VecDeque::from([3, 4, 5]));
    }
}
