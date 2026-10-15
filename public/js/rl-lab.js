/**
 * NeuroMount Reinforcement Learning & NeuroEvolution Lab
 * آزمایشگاه «سوار کردن مغز هوش مصنوعی» روی ماشین خودران و ربات مسیریاب
 */

class AutonomousCarLab {
  constructor(canvas, onUpdateCallback) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.onUpdate = onUpdateCallback || (() => {});

    this.width = 760;
    this.height = 460;

    // وضعیت شبیه‌سازی
    this.isRunning = false;
    this.simSpeed = 1;
    this.populationSize = 18;
    this.mutationRate = 0.18;
    this.generation = 1;
    this.frameCounter = 0;
    this.maxFramesPerGen = 520;
    this.bestFitnessEver = 0;
    this.lapsCompleted = 0;
    this.isBrainMounted = true;
    this.mountedBrainName = 'مغز عصبی نسل ۱ (خام)';

    this.fitnessHistory = [];
    this.customObstacles = [
      { x: 390, y: 86, r: 15 },
      { x: 540, y: 368, r: 16 }
    ];

    this.sensorAngles = [-1.25, -0.8, -0.38, 0, 0.38, 0.8, 1.25];
    this.sensorRange = 135;

    this.buildTrack();
    this.championBrain = new NeuralNetwork([8, 12, 8, 2], {
      activation: 'tanh',
      outputActivation: 'tanh'
    });
    this.cars = [];
    this.spawnGeneration(true);
    this.setupMouseInteraction();
  }

  buildTrack() {
    // مسیر پیست بیضویِ زاویه‌دار با پیچ‌های چالشی
    this.centerPath = [
      { x: 190, y: 92 },
      { x: 380, y: 78 },
      { x: 575, y: 92 },
      { x: 665, y: 175 },
      { x: 665, y: 295 },
      { x: 570, y: 375 },
      { x: 380, y: 382 },
      { x: 190, y: 372 },
      { x: 102, y: 290 },
      { x: 102, y: 170 }
    ];

    this.trackHalfWidth = 52;
    this.outerWalls = [];
    this.innerWalls = [];
    this.checkpoints = [];

    const cx = 384;
    const cy = 230;

    const outerPts = [];
    const innerPts = [];

    for (let i = 0; i < this.centerPath.length; i++) {
      const p = this.centerPath[i];
      const next = this.centerPath[(i + 1) % this.centerPath.length];

      // درونی‌سازی نقاط میانی برای چک‌پوینت‌های بیشتر (۲۰ چک‌پوینت)
      for (let sub = 0; sub < 2; sub++) {
        const t = sub * 0.5;
        const mx = p.x + (next.x - p.x) * t;
        const my = p.y + (next.y - p.y) * t;
        const dx = mx - cx;
        const dy = my - cy;
        const len = Math.hypot(dx, dy) || 1;
        const nx = dx / len;
        const ny = dy / len;

        const outP = { x: mx + nx * this.trackHalfWidth, y: my + ny * this.trackHalfWidth };
        const inP = { x: mx - nx * this.trackHalfWidth, y: my - ny * this.trackHalfWidth };

        if (sub === 0) {
          outerPts.push(outP);
          innerPts.push(inP);
        }
        this.checkpoints.push({ x1: inP.x, y1: inP.y, x2: outP.x, y2: outP.y, cx: mx, cy: my });
      }
    }

    for (let i = 0; i < outerPts.length; i++) {
      const next = (i + 1) % outerPts.length;
      this.outerWalls.push({
        x1: outerPts[i].x,
        y1: outerPts[i].y,
        x2: outerPts[next].x,
        y2: outerPts[next].y
      });
      this.innerWalls.push({
        x1: innerPts[i].x,
        y1: innerPts[i].y,
        x2: innerPts[next].x,
        y2: innerPts[next].y
      });
    }

    this.outerPolygon = outerPts;
    this.innerPolygon = innerPts;
    this.allWallSegments = [...this.outerWalls, ...this.innerWalls];
  }

  setupMouseInteraction() {
    this.canvas.addEventListener('click', e => {
      const rect = this.canvas.getBoundingClientRect();
      const scaleX = this.width / rect.width;
      const scaleY = this.height / rect.height;
      const x = (e.clientX - rect.left) * scaleX;
      const y = (e.clientY - rect.top) * scaleY;

      // اگر نزدیک مانع موجود کلیک شد، حذفش کن؛ وگرنه مانع جدید بگذار
      const existingIdx = this.customObstacles.findIndex(obs => Math.hypot(obs.x - x, obs.y - y) < obs.r + 12);
      if (existingIdx !== -1) {
        this.customObstacles.splice(existingIdx, 1);
      } else {
        this.customObstacles.push({ x, y, r: 15 });
      }
      this.render();
    });
  }

  createCar(brain) {
    return {
      x: 245,
      y: 86,
      angle: -0.05,
      speed: 2.2,
      alive: true,
      fitness: 0,
      checkpointsPassed: 0,
      nextCheckpoint: 2,
      timeAlive: 0,
      sensorReadings: new Array(this.sensorAngles.length).fill(0),
      sensorHits: [],
      brain: brain ? brain.clone() : new NeuralNetwork([8, 12, 8, 2]),
      isElite: false
    };
  }

  spawnGeneration(isInitial = false) {
    this.frameCounter = 0;
    const newCars = [];

    if (isInitial || this.cars.length === 0) {
      for (let i = 0; i < this.populationSize; i++) {
        const b = this.championBrain.clone();
        if (i > 0) b.mutate(0.6, 0.7);
        newCars.push(this.createCar(b));
      }
      newCars[0].isElite = true;
      this.cars = newCars;
      return;
    }

    // مرتب‌سازی نسل قبلی بر اساس شایستگی (Fitness)
    this.cars.sort((a, b) => b.fitness - a.fitness);
    const bestCar = this.cars[0];
    const secondBest = this.cars[1] || bestCar;

    if (bestCar.fitness > this.bestFitnessEver) {
      this.bestFitnessEver = Math.round(bestCar.fitness);
      this.championBrain = bestCar.brain.clone();
    }

    this.fitnessHistory.push({
      generation: this.generation,
      value: Math.round(bestCar.fitness)
    });
    if (this.fitnessHistory.length > 40) this.fitnessHistory.shift();

    this.generation++;
    this.mountedBrainName = `مغز خودآموز (نسل ${this.generation})`;

    // ۱. حفظ بهترین مغز بدون تغییر (Elitism)
    const eliteCar = this.createCar(this.championBrain.clone());
    eliteCar.isElite = true;
    newCars.push(eliteCar);

    // ۲. تولید بقیه جمعیت با ترکیب ژنتیکی + گرادیان هدایت‌شده برای یادگیری ملموس و سریع
    for (let i = 1; i < this.populationSize; i++) {
      let childBrain;
      if (i % 3 === 0) {
        childBrain = NeuralNetwork.crossover(bestCar.brain, secondBest.brain);
      } else {
        childBrain = this.championBrain.clone();
      }
      const adaptiveStrength = Math.max(0.12, 0.42 - this.generation * 0.015);
      childBrain.mutate(this.mutationRate, adaptiveStrength);

      // تقویت سیناپس‌های واکنش به دیوار در نسل‌های جدیدتر (Reinforcement Guidance)
      this.applyReflexGuidance(childBrain, 2);
      newCars.push(this.createCar(childBrain));
    }

    this.cars = newCars;
  }

  /**
   * آموزش تقویتی کمکی روی وزن‌های مغز تا ماشین در چند نسل به صورت واقعی پیچیدن و فرار از مانع را یاد بگیرد
   */
  applyReflexGuidance(brain, steps = 3) {
    const patterns = [
      // مانع در سمت چپ -> بپیچ به راست
      { in: [0.85, 0.75, 0.4, 0.1, 0.0, 0.0, 0.0, 0.6], out: [0.75, 0.4] },
      // مانع در سمت راست -> بپیچ به چپ
      { in: [0.0, 0.0, 0.0, 0.1, 0.4, 0.75, 0.85, 0.6], out: [-0.75, 0.4] },
      // مسیر باز در روبرو -> مستقیم و سریع برو
      { in: [0.1, 0.1, 0.05, 0.0, 0.05, 0.1, 0.1, 0.5], out: [0.0, 0.9] },
      // مانع مستقیم روبرو و چپ بسته -> شدید به راست
      { in: [0.7, 0.8, 0.9, 0.9, 0.2, 0.1, 0.0, 0.5], out: [0.95, -0.2] },
      // مانع مستقیم روبرو و راست بسته -> شدید به چپ
      { in: [0.0, 0.1, 0.2, 0.9, 0.9, 0.8, 0.7, 0.5], out: [-0.95, -0.2] }
    ];
    for (let s = 0; s < steps; s++) {
      for (const p of patterns) {
        brain.trainStep(p.in, p.out, 0.045);
      }
    }
  }

  /**
   * سوار کردن مغزِ استاد (از پیش آموزش‌دیده کامل) روی ماشین‌ها
   */
  mountMasterBrain() {
    const master = new NeuralNetwork([8, 12, 8, 2], {
      activation: 'tanh',
      outputActivation: 'tanh'
    });
    for (let i = 0; i < 65; i++) {
      this.applyReflexGuidance(master, 4);
    }
    this.championBrain = master;
    this.generation = Math.max(this.generation, 35);
    this.mountedBrainName = '🏆 مغز قهرمان پیست (Master Brain)';
    this.isBrainMounted = true;

    this.cars = [];
    for (let i = 0; i < this.populationSize; i++) {
      const b = master.clone();
      if (i > 0) b.mutate(0.08, 0.08);
      const c = this.createCar(b);
      if (i === 0) c.isElite = true;
      this.cars.push(c);
    }
    this.frameCounter = 0;
    this.onUpdate(this.getTelemetry());
    this.render();
  }

  /**
   * ریست کردن مغز به حالت کاملاً خام (نسل ۱)
   */
  resetToRawBrain() {
    this.generation = 1;
    this.bestFitnessEver = 0;
    this.lapsCompleted = 0;
    this.fitnessHistory = [];
    this.championBrain = new NeuralNetwork([8, 12, 8, 2]);
    this.mountedBrainName = 'مغز عصبی نسل ۱ (خام)';
    this.isBrainMounted = true;
    this.cars = [];
    this.spawnGeneration(true);
    this.onUpdate(this.getTelemetry());
    this.render();
  }

  /**
   * آموزش سریع چند نسل در پس‌زمینه (Fast-Forward Training)
   */
  fastForwardGenerations(numGens = 8) {
    for (let g = 0; g < numGens; g++) {
      for (let f = 0; f < 260; f++) {
        this.stepPhysics();
        if (this.cars.every(c => !c.alive)) break;
      }
      this.spawnGeneration(false);
    }
    this.onUpdate(this.getTelemetry());
    this.render();
  }

  stepPhysics() {
    this.frameCounter++;
    let aliveCount = 0;

    for (const car of this.cars) {
      if (!car.alive) continue;
      aliveCount++;
      car.timeAlive++;

      // ۱. محاسبه برخورد اشعه‌های لیزر سنسورها با دیوارها و موانع
      car.sensorHits = [];
      for (let s = 0; s < this.sensorAngles.length; s++) {
        const rayAngle = car.angle + this.sensorAngles[s];
        const rx = car.x + Math.cos(rayAngle) * this.sensorRange;
        const ry = car.y + Math.sin(rayAngle) * this.sensorRange;

        let minDist = this.sensorRange;
        let hitPt = { x: rx, y: ry };

        for (const seg of this.allWallSegments) {
          const hit = this.raySegmentIntersect(car.x, car.y, rx, ry, seg.x1, seg.y1, seg.x2, seg.y2);
          if (hit && hit.dist < minDist) {
            minDist = hit.dist;
            hitPt = { x: hit.x, y: hit.y };
          }
        }

        for (const obs of this.customObstacles) {
          const hit = this.rayCircleIntersect(car.x, car.y, rx, ry, obs.x, obs.y, obs.r);
          if (hit && hit.dist < minDist) {
            minDist = hit.dist;
            hitPt = { x: hit.x, y: hit.y };
          }
        }

        // مقدار ۱ یعنی مانع خیلی نزدیک است، ۰ یعنی مسیر کاملاً باز است
        const closeness = 1.0 - minDist / this.sensorRange;
        car.sensorReadings[s] = closeness;
        car.sensorHits.push({ x: hitPt.x, y: hitPt.y, closeness });
      }

      // ۲. اجرای شبکه عصبی سوارشده روی ماشین
      if (this.isBrainMounted) {
        const inputs = [...car.sensorReadings, car.speed / 4.5];
        const [steerOut, throttleOut] = car.brain.forward(inputs);

        car.angle += steerOut * 0.085;
        const targetSpeed = 2.0 + (throttleOut + 1) * 1.15;
        car.speed = car.speed * 0.88 + targetSpeed * 0.12;
      } else {
        // اگر مغز جدا شده باشد، ماشین مستقیم می‌رود و به دیوار می‌خورد
        car.speed = 2.5;
      }

      car.x += Math.cos(car.angle) * car.speed;
      car.y += Math.sin(car.angle) * car.speed;

      // ۳. بررسی عبور از چک‌پوینت بعدی
      const cp = this.checkpoints[car.nextCheckpoint];
      const distToCp = Math.hypot(car.x - cp.cx, car.y - cp.cy);
      if (distToCp < this.trackHalfWidth * 0.95) {
        car.checkpointsPassed++;
        car.fitness += 45 + car.speed * 5;
        car.nextCheckpoint = (car.nextCheckpoint + 1) % this.checkpoints.length;
        if (car.checkpointsPassed > 0 && car.checkpointsPassed % this.checkpoints.length === 0) {
          this.lapsCompleted++;
          car.fitness += 250;
        }
      }

      car.fitness += car.speed * 0.18;

      // ۴. بررسی تصادف با دیوار یا مانع
      if (this.checkCollision(car)) {
        car.alive = false;
      }
    }

    if (aliveCount === 0 || this.frameCounter >= this.maxFramesPerGen) {
      this.spawnGeneration(false);
    }
  }

  checkCollision(car) {
    // اگر هر یک از سنسورها به فاصله خیلی نزدیک (< 9px) رسید یا با دایره مانع برخورد کرد
    for (const seg of this.allWallSegments) {
      const d = this.pointToSegmentDist(car.x, car.y, seg.x1, seg.y1, seg.x2, seg.y2);
      if (d < 8.5) return true;
    }
    for (const obs of this.customObstacles) {
      if (Math.hypot(car.x - obs.x, car.y - obs.y) < obs.r + 7) return true;
    }
    return false;
  }

  raySegmentIntersect(x1, y1, x2, y2, x3, y3, x4, y4) {
    const denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if (Math.abs(denom) < 1e-6) return null;
    const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    const u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;
    if (t >= 0 && t <= 1 && u >= 0 && u <= 1) {
      const ix = x1 + t * (x2 - x1);
      const iy = y1 + t * (y2 - y1);
      return { x: ix, y: iy, dist: Math.hypot(ix - x1, iy - y1) };
    }
    return null;
  }

  rayCircleIntersect(x1, y1, x2, y2, cx, cy, r) {
    const dx = x2 - x1;
    const dy = y2 - y1;
    const fx = x1 - cx;
    const fy = y1 - cy;
    const a = dx * dx + dy * dy;
    const b = 2 * (fx * dx + fy * dy);
    const c = fx * fx + fy * fy - r * r;
    let discriminant = b * b - 4 * a * c;
    if (discriminant < 0) return null;
    discriminant = Math.sqrt(discriminant);
    const t = (-b - discriminant) / (2 * a);
    if (t >= 0 && t <= 1) {
      const ix = x1 + t * dx;
      const iy = y1 + t * dy;
      return { x: ix, y: iy, dist: Math.hypot(ix - x1, iy - y1) };
    }
    return null;
  }

  pointToSegmentDist(px, py, x1, y1, x2, y2) {
    const l2 = (x2 - x1) ** 2 + (y2 - y1) ** 2;
    if (l2 === 0) return Math.hypot(px - x1, py - y1);
    let t = ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2;
    t = Math.max(0, Math.min(1, t));
    return Math.hypot(px - (x1 + t * (x2 - x1)), py - (y1 + t * (y2 - y1)));
  }

  getLeadCar() {
    const aliveCars = this.cars.filter(c => c.alive);
    const pool = aliveCars.length > 0 ? aliveCars : this.cars;
    if (pool.length === 0) return null;
    return pool.reduce((best, c) => (c.fitness > best.fitness ? c : best), pool[0]);
  }

  getTelemetry() {
    const lead = this.getLeadCar();
    const aliveCount = this.cars.filter(c => c.alive).length;
    const currentBest = lead ? Math.round(lead.fitness) : 0;
    return {
      generation: this.generation,
      aliveCount,
      populationSize: this.populationSize,
      currentFitness: currentBest,
      bestFitnessEver: Math.max(this.bestFitnessEver, currentBest),
      lapsCompleted: this.lapsCompleted,
      mountedBrainName: this.mountedBrainName,
      isBrainMounted: this.isBrainMounted,
      leadBrain: lead ? lead.brain : this.championBrain,
      fitnessHistory: this.fitnessHistory
    };
  }

  render() {
    const ctx = this.ctx;
    const dpr = window.devicePixelRatio || 1;
    const rect = this.canvas.getBoundingClientRect();
    const displayW = rect.width || this.width;
    const displayH = rect.height || this.height;

    if (
      this.canvas.width !== Math.floor(displayW * dpr) ||
      this.canvas.height !== Math.floor(displayH * dpr)
    ) {
      this.canvas.width = Math.floor(displayW * dpr);
      this.canvas.height = Math.floor(displayH * dpr);
    }

    ctx.save();
    ctx.scale((displayW * dpr) / this.width, (displayH * dpr) / this.height);

    // ۱. پس‌زمینه سایبری تیره
    ctx.fillStyle = '#070b14';
    ctx.fillRect(0, 0, this.width, this.height);

    // شبکه شطرنجی کمرنگ
    ctx.strokeStyle = 'rgba(30, 41, 59, 0.45)';
    ctx.lineWidth = 1;
    for (let x = 0; x < this.width; x += 38) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, this.height);
      ctx.stroke();
    }
    for (let y = 0; y < this.height; y += 38) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(this.width, y);
      ctx.stroke();
    }

    // ۲. رسم آسفالت پیست
    ctx.beginPath();
    this.outerPolygon.forEach((p, idx) => {
      if (idx === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.closePath();
    ctx.fillStyle = '#131c2e';
    ctx.fill();

    // جزیره وسط پیست
    ctx.beginPath();
    this.innerPolygon.forEach((p, idx) => {
      if (idx === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.closePath();
    ctx.fillStyle = '#070b14';
    ctx.fill();

    // ۳. خطوط چک‌پوینت‌ها
    for (let i = 0; i < this.checkpoints.length; i++) {
      const cp = this.checkpoints[i];
      ctx.beginPath();
      ctx.moveTo(cp.x1, cp.y1);
      ctx.lineTo(cp.x2, cp.y2);
      ctx.strokeStyle = i === 1 ? 'rgba(16, 185, 129, 0.55)' : 'rgba(56, 189, 248, 0.12)';
      ctx.lineWidth = i === 1 ? 3 : 1;
      ctx.stroke();
    }

    // ۴. دیوارهای بیرونی و درونی با درخشش نئونی
    ctx.strokeStyle = '#38bdf8';
    ctx.lineWidth = 2.6;
    for (const seg of this.outerWalls) {
      ctx.beginPath();
      ctx.moveTo(seg.x1, seg.y1);
      ctx.lineTo(seg.x2, seg.y2);
      ctx.stroke();
    }

    ctx.strokeStyle = '#818cf8';
    for (const seg of this.innerWalls) {
      ctx.beginPath();
      ctx.moveTo(seg.x1, seg.y1);
      ctx.lineTo(seg.x2, seg.y2);
      ctx.stroke();
    }

    // ۵. رسم موانع تعاملی (قابل کاشت با کلیک کاربر)
    for (const obs of this.customObstacles) {
      const grad = ctx.createRadialGradient(obs.x, obs.y, 2, obs.x, obs.y, obs.r * 1.5);
      grad.addColorStop(0, 'rgba(244, 63, 94, 0.9)');
      grad.addColorStop(1, 'rgba(244, 63, 94, 0)');
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(obs.x, obs.y, obs.r * 1.5, 0, Math.PI * 2);
      ctx.fill();

      ctx.beginPath();
      ctx.arc(obs.x, obs.y, obs.r, 0, Math.PI * 2);
      ctx.fillStyle = '#e11d48';
      ctx.fill();
      ctx.lineWidth = 2;
      ctx.strokeStyle = '#fda4af';
      ctx.stroke();
    }

    const leadCar = this.getLeadCar();

    // ۶. رسم اشعه‌های لیزر سنسور برای ماشین پیشتاز
    if (leadCar && leadCar.alive && leadCar.sensorHits.length > 0) {
      for (const hit of leadCar.sensorHits) {
        ctx.beginPath();
        ctx.moveTo(leadCar.x, leadCar.y);
        ctx.lineTo(hit.x, hit.y);
        const isDanger = hit.closeness > 0.55;
        ctx.strokeStyle = isDanger ? 'rgba(244, 63, 94, 0.75)' : 'rgba(34, 211, 238, 0.5)';
        ctx.lineWidth = 1.4;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(hit.x, hit.y, 3, 0, Math.PI * 2);
        ctx.fillStyle = isDanger ? '#f43f5e' : '#22d3ee';
        ctx.fill();
      }
    }

    // ۷. رسم تمام ماشین‌های جمعیت
    for (const car of this.cars) {
      const isLead = car === leadCar;
      ctx.save();
      ctx.translate(car.x, car.y);
      ctx.rotate(car.angle);

      if (!car.alive) {
        ctx.globalAlpha = 0.18;
        ctx.fillStyle = '#64748b';
      } else if (isLead) {
        ctx.globalAlpha = 1.0;
        ctx.fillStyle = '#10b981';
        ctx.shadowColor = '#10b981';
        ctx.shadowBlur = 12;
      } else {
        ctx.globalAlpha = 0.42;
        ctx.fillStyle = '#38bdf8';
      }

      // بدنه ماشین
      ctx.beginPath();
      ctx.roundRect(-11, -6, 22, 12, 3);
      ctx.fill();

      // شیشه جلو و چراغ‌ها
      ctx.fillStyle = isLead ? '#ecfdf5' : '#0f172a';
      ctx.fillRect(2, -4, 6, 8);

      ctx.restore();
    }

    // راهنمای گوشه تصویر
    ctx.fillStyle = 'rgba(15, 23, 42, 0.82)';
    ctx.fillRect(14, 14, 265, 30);
    ctx.fillStyle = '#cbd5e1';
    ctx.font = '11.5px Vazirmatn, Tahoma, sans-serif';
    ctx.fillText('💡 روی هر نقطه از پیست کلیک کن تا مانع بکاری/برداری!', 24, 33);

    ctx.restore();
  }
}

window.AutonomousCarLab = AutonomousCarLab;
