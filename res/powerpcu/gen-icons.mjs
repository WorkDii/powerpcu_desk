// Regenerates res/powerpcu/out/ from the SVG sources. Dev-time only; CI uses the
// committed output. Needs @resvg/resvg-js installed outside the repo:
//   npm --prefix <dir> i @resvg/resvg-js@2.6.2
//   POWERPCU_ICON_TOOLS=<dir> node res/powerpcu/gen-icons.mjs
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const tools = process.env.POWERPCU_ICON_TOOLS;
if (!tools) {
  console.error('Set POWERPCU_ICON_TOOLS to the directory holding node_modules/@resvg/resvg-js');
  process.exit(1);
}
const { Resvg } = createRequire(path.join(tools, 'package.json'))('@resvg/resvg-js');

const here = path.dirname(fileURLToPath(import.meta.url));
const out = path.join(here, 'out');
fs.mkdirSync(out, { recursive: true });
const read = (name) => fs.readFileSync(path.join(here, name), 'utf8');
const big = read('logo.svg');
const small = read('logo-small.svg');
const wordmark = read('wordmark.svg');

// Strokes drawn for 24 px and up turn to mush below that.
const sourceFor = (size) => (size <= 24 ? small : big);

function render(svg, width, loadSystemFonts = false) {
  return new Resvg(svg, {
    fitTo: { mode: 'width', value: width },
    font: { loadSystemFonts },
  }).render();
}

// 32-bit BMP entries below 256 px and PNG at 256 px: LoadImageW only promises
// PNG-compressed entries at 256.
function bmpEntry(size) {
  const rgba = render(sourceFor(size), size).pixels;
  const maskRow = Math.ceil(size / 32) * 4;
  const header = Buffer.alloc(40);
  header.writeUInt32LE(40, 0);
  header.writeInt32LE(size, 4);
  header.writeInt32LE(size * 2, 8);
  header.writeUInt16LE(1, 12);
  header.writeUInt16LE(32, 14);
  header.writeUInt32LE(size * size * 4 + maskRow * size, 20);
  const pixels = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const src = (y * size + x) * 4;
      const dst = ((size - 1 - y) * size + x) * 4;
      pixels[dst] = rgba[src + 2];
      pixels[dst + 1] = rgba[src + 1];
      pixels[dst + 2] = rgba[src];
      pixels[dst + 3] = rgba[src + 3];
    }
  }
  return Buffer.concat([header, pixels, Buffer.alloc(maskRow * size)]);
}

function writeIco(name, sizes) {
  const images = sizes.map((s) => (s >= 256 ? render(big, s).asPng() : bmpEntry(s)));
  const dir = Buffer.alloc(6 + 16 * sizes.length);
  dir.writeUInt16LE(1, 2);
  dir.writeUInt16LE(sizes.length, 4);
  let offset = dir.length;
  sizes.forEach((s, i) => {
    const e = 6 + 16 * i;
    dir.writeUInt8(s >= 256 ? 0 : s, e);
    dir.writeUInt8(s >= 256 ? 0 : s, e + 1);
    dir.writeUInt16LE(1, e + 4);
    dir.writeUInt16LE(32, e + 6);
    dir.writeUInt32LE(images[i].length, e + 8);
    dir.writeUInt32LE(offset, e + 12);
    offset += images[i].length;
  });
  fs.writeFileSync(path.join(out, name), Buffer.concat([dir, ...images]));
}

writeIco('app_icon.ico', [16, 20, 24, 32, 40, 48, 64, 256]);
writeIco('icon.ico', [16, 24, 32, 48, 64, 128, 256]);
writeIco('tray-icon.ico', [16, 20, 24, 32]);
writeIco('assets-icon.ico', [16, 24, 32, 48, 64, 128, 256]);
// The tray and loadIcon() show assets/icon.png at 16-24 px, so it keeps the bold strokes.
fs.writeFileSync(path.join(out, 'assets-icon.png'), render(small, 256).asPng());
fs.copyFileSync(path.join(here, 'logo.svg'), path.join(out, 'icon.svg'));
fs.writeFileSync(path.join(out, 'logo.png'), render(wordmark, 600, true).asPng());
const darkWordmark = wordmark.replace(
  /(<text id="wordmark-text"[^>]*fill=")#004c28(")/,
  '$1#e6f4ec$2',
);
fs.writeFileSync(path.join(out, 'logo_dark.png'), render(darkWordmark, 600, true).asPng());
console.log('wrote', fs.readdirSync(out).join(', '));
