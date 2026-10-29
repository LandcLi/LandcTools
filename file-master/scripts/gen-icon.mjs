// 生成应用图标源图（无第三方依赖，手写 PNG 编码）。
// 用法：node scripts/gen-icon.mjs [输出路径]
// 设计：圆角方形渐变底 + 白色文件卡 + 折角 + 向下批量箭头

import { deflateSync } from 'node:zlib'
import { writeFileSync, mkdirSync } from 'node:fs'
import { dirname } from 'node:path'

const SIZE = 1024
const SS = 3 // 每像素 3x3 超采样做抗锯齿

// ---------- PNG 编码 ----------
const crcTable = (() => {
  const t = new Uint32Array(256)
  for (let n = 0; n < 256; n++) {
    let c = n
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
    t[n] = c >>> 0
  }
  return t
})()

function crc32(buf) {
  let c = 0xffffffff
  for (let i = 0; i < buf.length; i++) c = crcTable[(c ^ buf[i]) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}

function chunk(type, data) {
  const len = Buffer.alloc(4)
  len.writeUInt32BE(data.length)
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data])
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(body))
  return Buffer.concat([len, body, crc])
}

function encodePng(width, height, rgba) {
  const ihdr = Buffer.alloc(13)
  ihdr.writeUInt32BE(width, 0)
  ihdr.writeUInt32BE(height, 4)
  ihdr[8] = 8 // bit depth
  ihdr[9] = 6 // RGBA
  ihdr[10] = 0
  ihdr[11] = 0
  ihdr[12] = 0

  // 每行前置 filter byte 0
  const raw = Buffer.alloc((width * 4 + 1) * height)
  for (let y = 0; y < height; y++) {
    raw[y * (width * 4 + 1)] = 0
    rgba.copy(raw, y * (width * 4 + 1) + 1, y * width * 4, (y + 1) * width * 4)
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ])
}

// ---------- 几何工具 ----------
function roundedRectSdf(px, py, cx, cy, hw, hh, r) {
  const qx = Math.abs(px - cx) - hw + r
  const qy = Math.abs(py - cy) - hh + r
  const ax = Math.max(qx, 0)
  const ay = Math.max(qy, 0)
  return Math.sqrt(ax * ax + ay * ay) + Math.min(Math.max(qx, qy), 0) - r
}

function segmentDist(px, py, ax, ay, bx, by) {
  const vx = bx - ax
  const vy = by - ay
  const wx = px - ax
  const wy = py - ay
  const len2 = vx * vx + vy * vy
  let t = len2 === 0 ? 0 : (wx * vx + wy * vy) / len2
  t = Math.max(0, Math.min(1, t))
  const dx = px - (ax + t * vx)
  const dy = py - (ay + t * vy)
  return Math.sqrt(dx * dx + dy * dy)
}

function lerp(a, b, t) {
  return a + (b - a) * t
}

const C = {
  gradA: [79, 70, 229], // #4F46E5 indigo
  gradB: [168, 85, 247], // #A855F7 purple
  white: [255, 255, 255],
  fold: [226, 222, 255],
  line: [109, 101, 220],
  arrow: [99, 91, 255],
}

// 形状描述：返回该点的颜色贡献 [r,g,b,a]
function sample(x, y) {
  // 底板
  if (roundedRectSdf(x, y, 512, 512, 448, 448, 210) < 0) {
    const t = Math.min(1, Math.max(0, (x + y) / (2 * SIZE)))
    const r = lerp(C.gradA[0], C.gradB[0], t)
    const g = lerp(C.gradA[1], C.gradB[1], t)
    const b = lerp(C.gradA[2], C.gradB[2], t)

    // 文件卡：中心 (512,440)，半宽 214，半高 240，圆角 46 → 占 298~726 / 200~680
    if (roundedRectSdf(x, y, 512, 440, 214, 240, 46) < 0) {
      // 右上折角：(636,200) (726,200) (726,290) 三角形
      if (x >= 636 && y >= 200 && x <= 726 && y <= 290 && x - 636 >= y - 200) {
        return [C.fold[0], C.fold[1], C.fold[2], 255]
      }
      // 两条“文件名”色块
      if (y > 350 && y < 392 && x > 362 && x < 662) return [...C.line, 255]
      if (y > 438 && y < 480 && x > 362 && x < 572) return [...C.line, 255]
      return [C.white[0], C.white[1], C.white[2], 255]
    }

    // 向下箭头：完整落在卡片下方 706~846
    const shaft = segmentDist(x, y, 512, 706, 512, 842)
    const headL = segmentDist(x, y, 458, 790, 512, 846)
    const headR = segmentDist(x, y, 566, 790, 512, 846)
    if (Math.min(shaft, headL, headR) < 23) return [255, 255, 255, 255]

    return [r, g, b, 255]
  }
  return [0, 0, 0, 0]
}

// ---------- 渲染 ----------
const rgba = Buffer.alloc(SIZE * SIZE * 4)
const inv = 1 / (SS * SS)
for (let y = 0; y < SIZE; y++) {
  for (let x = 0; x < SIZE; x++) {
    let r = 0
    let g = 0
    let b = 0
    let a = 0
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const px = x + (sx + 0.5) / SS
        const py = y + (sy + 0.5) / SS
        const c = sample(px, py)
        const al = c[3] / 255
        r += c[0] * al
        g += c[1] * al
        b += c[2] * al
        a += c[3]
      }
    }
    const alpha = a * inv
    const idx = (y * SIZE + x) * 4
    if (alpha > 0) {
      const k = 255 / alpha
      rgba[idx] = Math.min(255, Math.round(r * inv * k))
      rgba[idx + 1] = Math.min(255, Math.round(g * inv * k))
      rgba[idx + 2] = Math.min(255, Math.round(b * inv * k))
    }
    rgba[idx + 3] = Math.round(alpha)
  }
}

const out = process.argv[2] || 'src-tauri/app-icon.png'
mkdirSync(dirname(out), { recursive: true })
writeFileSync(out, encodePng(SIZE, SIZE, rgba))
console.log(`图标已生成：${out} (${SIZE}x${SIZE})`)
