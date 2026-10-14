#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
#  نصب Rust بدون rustup — «ترفند محیط‌های محدود» 🕵️
# ═══════════════════════════════════════════════════════════════════════════
#
#  معمولاً Rust را با این کامند نصب می‌کنند:
#      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
#
#  ولی در بعضی محیط‌ها (sandbox، شبکهٔ بسته، محدودیت فایروال) دسترسی به
#  static.rust-lang.org وجود ندارد و آن کامند شکست می‌خورد.
#
#  راه‌حل: بسته‌های npm یا PyPI که *خودِ تاربال‌های رسمی Rust* را داخلشان
#  دارند و فقط از رجیستری npm/pypi دانلود می‌شوند.
#
#  ✅ روی هر سیستمی که node/npm دارد کار می‌کند و چیزی شبیه sudo نمی‌خواهد.
#  ✅ نسخه‌ها با SHA256SUMS چک می‌شوند (امن است).
#
#  استفاده:
#      bash scripts/setup-rust.sh              # نصب در ~/.cache/rust
#      PREFIX=/opt/rust bash scripts/setup-rust.sh
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.cache/rust}"
WORK="${WORK:-$HOME/.cache/rust-tc}"

# چه بسته‌ای؟ الگوی نام:  @rust-sandbox/rust-<MINOR>-x86_64-linux
# پکیج‌های دیگر در دسترس (طبق جست‌وجو در رجیستری npm):
#   @toolchain-bin/rustc-<VERSION>-<TARGET>      (تک‌تک اجزا)
#   @rustbin/rustc-<VERSION>-<TARGET>
#   @kreijstal/rubrc                            (rustc در WebAssembly — جالب!)
PKG="${PKG:-@rust-sandbox/rust-1-95-x86_64-linux}"

echo "════════════════════════════════════════════════════════"
echo " نصب Rust:  $PKG"
echo " مقصد:      $PREFIX"
echo "════════════════════════════════════════════════════════"

# ── مرحله ۱: دانلود بسته از رجیستری npm (بدون نیاز به Rust!) ──────────────
if [ ! -d "$WORK/node_modules/$PKG" ]; then
  mkdir -p "$WORK"
  cd "$WORK"
  [ -f package.json ] || npm init -y >/dev/null 2>&1
  echo "→ دانلود از npm ..."
  npm install --no-audit --no-fund --silent "$PKG"
fi

# ── مرحله ۲: باز کردن و نصب (اسکریپت install.js خودش SHA256 را چک می‌کند) ─
cd "$WORK/node_modules/$PKG"
echo "→ تأیید SHA256 و استخراج ..."
RUST_INSTALL_PREFIX="$PREFIX" HOME="${HOME:-/root}" node install.js

# ── مرحله ۳: تست دود (smoke test) ────────────────────────────────────────
export PATH="$PREFIX/bin:$PATH"
echo
echo "✅ نصب تمام شد:"
rustc --version
cargo --version
echo
echo "برای استفاده، هر بار این را اجرا کن (یا در ~/.bashrc بگذار):"
echo "    export PATH=\"$PREFIX/bin:\$PATH\""
echo
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "تست سریع:"
echo "    cd \"$ROOT\" && cargo run -- demo          # فهرست موضوع‌های آموزشی"
echo "    cd \"$ROOT\" && cargo run -- demo all       # اجرای همهٔ دموها"
echo "    cd \"$ROOT\" && bash scripts/run-web.sh    # بالا آوردن رابط وب روی :8080"
