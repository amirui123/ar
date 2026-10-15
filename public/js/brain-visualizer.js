/**
 * NeuroMount Brain & Metrics Visualizer
 * رسم گراف شبکه عصبی زنده (نورون‌ها، سیناپس‌ها، پالس‌های یادگیری) و نمودارهای Loss / Reward
 */

class BrainVisualizer {
  static renderNetwork(canvas, network, config = {}) {
    if (!canvas || !network) return;
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = rect.width || canvas.width || 480;
    const height = rect.height || canvas.height || 300;

    if (canvas.width !== Math.floor(width * dpr) || canvas.height !== Math.floor(height * dpr)) {
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
    }

    ctx.save();
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    const layers = network.layerSizes;
    const numLayers = layers.length;
    const padX = config.padX || 65;
    const padY = config.padY || 28;
    const usableW = Math.max(100, width - padX * 2);
    const usableH = Math.max(80, height - padY * 2);

    const coords = [];
    for (let l = 0; l < numLayers; l++) {
      const count = layers[l];
      const x = padX + (l / Math.max(1, numLayers - 1)) * usableW;
      const layerCoords = [];
      for (let n = 0; n < count; n++) {
        const y = count === 1
          ? height / 2
          : padY + (n / (count - 1)) * usableH;
        layerCoords.push({ x, y });
      }
      coords.push(layerCoords);
    }

    // ۱. رسم سیناپس‌ها (وزن‌ها)
    for (let l = 0; l < network.weights.length; l++) {
      const W = network.weights[l];
      const inActs = network.lastActivations[l] || [];
      for (let j = 0; j < W.length; j++) {
        for (let i = 0; i < W[j].length; i++) {
          const w = W[j][i];
          const act = inActs[i] ?? 0;
          const signalStrength = Math.min(1, Math.abs(w * (0.35 + Math.abs(act))));
          if (signalStrength < 0.03) continue;

          const p1 = coords[l][i];
          const p2 = coords[l + 1][j];

          ctx.beginPath();
          ctx.moveTo(p1.x, p1.y);
          ctx.lineTo(p2.x, p2.y);

          const alpha = Math.min(0.85, 0.08 + signalStrength * 0.65);
          if (w >= 0) {
            ctx.strokeStyle = `rgba(34, 211, 238, ${alpha})`; // فیروزه‌ای برای وزن مثبت
          } else {
            ctx.strokeStyle = `rgba(244, 63, 94, ${alpha})`;  // قرمز/مرجانی برای وزن منفی
          }
          ctx.lineWidth = Math.min(3.2, 0.4 + Math.abs(w) * 1.3);
          ctx.stroke();
        }
      }
    }

    // ۲. رسم نورون‌ها و مقدار فعال‌سازی لحظه‌ای
    const inputLabels = config.inputLabels || [];
    const outputLabels = config.outputLabels || [];

    for (let l = 0; l < numLayers; l++) {
      const acts = network.lastActivations[l] || [];
      const count = layers[l];
      const radius = count > 11 ? 8 : count > 8 ? 10 : 12;

      for (let n = 0; n < count; n++) {
        const { x, y } = coords[l][n];
        const val = acts[n] ?? 0;
        const norm = Math.max(-1, Math.min(1, val));

        // هاله نورانی دور نورون فعال
        if (Math.abs(norm) > 0.2) {
          const grad = ctx.createRadialGradient(x, y, 2, x, y, radius * 2.1);
          const glowColor = norm >= 0 ? '34, 211, 238' : '244, 63, 94';
          grad.addColorStop(0, `rgba(${glowColor}, ${Math.abs(norm) * 0.45})`);
          grad.addColorStop(1, `rgba(${glowColor}, 0)`);
          ctx.fillStyle = grad;
          ctx.beginPath();
          ctx.arc(x, y, radius * 2.1, 0, Math.PI * 2);
          ctx.fill();
        }

        // دایره اصلی نورون
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, Math.PI * 2);
        if (norm >= 0) {
          const intensity = Math.round(30 + norm * 180);
          ctx.fillStyle = `rgb(15, ${Math.min(230, intensity)}, ${Math.min(255, intensity + 30)})`;
        } else {
          const intensity = Math.round(30 + Math.abs(norm) * 190);
          ctx.fillStyle = `rgb(${Math.min(245, intensity + 20)}, 24, 55)`;
        }
        ctx.fill();

        ctx.lineWidth = 1.6;
        ctx.strokeStyle = Math.abs(norm) > 0.35 ? '#e2e8f0' : 'rgba(148, 163, 184, 0.5)';
        ctx.stroke();

        // مقدار عددی داخل نورون اگر سایز مناسب بود
        if (radius >= 10) {
          ctx.fillStyle = '#f8fafc';
          ctx.font = '600 8.5px monospace';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(val.toFixed(1), x, y);
        }

        // برچسب ورودی و خروجی
        ctx.font = '500 10.5px Vazirmatn, Tahoma, sans-serif';
        ctx.fillStyle = '#cbd5e1';
        if (l === 0 && inputLabels[n]) {
          ctx.textAlign = 'right';
          ctx.fillText(inputLabels[n], x - radius - 6, y);
        } else if (l === numLayers - 1 && outputLabels[n]) {
          ctx.textAlign = 'left';
          ctx.fillText(outputLabels[n], x + radius + 6, y);
        }
      }
    }

    ctx.restore();
  }

  static renderChart(canvas, dataPoints, options = {}) {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = rect.width || canvas.width || 460;
    const height = rect.height || canvas.height || 180;

    if (canvas.width !== Math.floor(width * dpr) || canvas.height !== Math.floor(height * dpr)) {
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
    }

    ctx.save();
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    const padLeft = 42;
    const padRight = 18;
    const padTop = 22;
    const padBottom = 26;
    const plotW = width - padLeft - padRight;
    const plotH = height - padTop - padBottom;

    // خطوط شبکه پس‌زمینه
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.14)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = padTop + (i / 4) * plotH;
      ctx.beginPath();
      ctx.moveTo(padLeft, y);
      ctx.lineTo(width - padRight, y);
      ctx.stroke();
    }

    if (!dataPoints || dataPoints.length === 0) {
      ctx.fillStyle = '#64748b';
      ctx.font = '12px Vazirmatn, Tahoma, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(options.emptyText || 'در انتظار شروع آموزش...', width / 2, height / 2);
      ctx.restore();
      return;
    }

    const values = dataPoints.map(d => d.value);
    const minVal = options.minValue !== undefined ? options.minValue : Math.min(...values, 0);
    const maxVal = options.maxValue !== undefined ? options.maxValue : Math.max(...values, minVal + 0.5);
    const range = Math.max(0.001, maxVal - minVal);

    // برچسب محور عمودی
    ctx.fillStyle = '#94a3b8';
    ctx.font = '10px monospace';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    ctx.fillText(maxVal.toFixed(1), padLeft - 6, padTop);
    ctx.fillText(((maxVal + minVal) / 2).toFixed(1), padLeft - 6, padTop + plotH / 2);
    ctx.fillText(minVal.toFixed(1), padLeft - 6, padTop + plotH);

    const color = options.color || '#22d3ee';
    const fillAlpha = options.fillColor || 'rgba(34, 211, 238, 0.16)';

    const pts = dataPoints.map((d, idx) => {
      const x = padLeft + (dataPoints.length === 1 ? plotW / 2 : (idx / (dataPoints.length - 1)) * plotW);
      const y = padTop + plotH - ((d.value - minVal) / range) * plotH;
      return { x, y };
    });

    // ناحیه رنگی زیر نمودار
    ctx.beginPath();
    ctx.moveTo(pts[0].x, padTop + plotH);
    for (const p of pts) ctx.lineTo(p.x, p.y);
    ctx.lineTo(pts[pts.length - 1].x, padTop + plotH);
    ctx.closePath();
    ctx.fillStyle = fillAlpha;
    ctx.fill();

    // خط اصلی نمودار
    ctx.beginPath();
    pts.forEach((p, idx) => {
      if (idx === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.strokeStyle = color;
    ctx.lineWidth = 2.4;
    ctx.stroke();

    // نقطه آخر
    const last = pts[pts.length - 1];
    ctx.beginPath();
    ctx.arc(last.x, last.y, 4, 0, Math.PI * 2);
    ctx.fillStyle = color;
    ctx.fill();

    ctx.restore();
  }
}

window.BrainVisualizer = BrainVisualizer;
