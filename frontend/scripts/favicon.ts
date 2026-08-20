import sharp from 'sharp';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const INPUT_SVG = path.resolve(__dirname, '../public/favicon.svg');
const PNG_OUTPUT_DIR = path.resolve(__dirname, '../public/icons');
const MANIFEST_PATH = path.resolve(__dirname, '../public/manifest.webmanifest');
const FAVICON_PATH = path.resolve(__dirname, '../public/favicon.ico');

const PNG_SIZES = [72, 96, 128, 144, 152, 192, 384, 512] as const;
const ICO_SIZES = [16, 32, 64] as const;

async function generateIcons(): Promise<void> {
  try {
    await fs.mkdir(PNG_OUTPUT_DIR, { recursive: true });

    // 1. Generate PNG icons
    await Promise.all(
      PNG_SIZES.map((size) =>
        sharp(INPUT_SVG)
          .resize(size, size)
          .toFile(path.join(PNG_OUTPUT_DIR, `icon-${size}x${size}.png`)),
      ),
    );

    // 2. Generate .ico favicon
    await createIcoFromPngs(
      await Promise.all(
        ICO_SIZES.map((size) => sharp(INPUT_SVG).resize(size, size).toFormat('png').toBuffer()),
      ),
    ).then((icoBuffer) => fs.writeFile(FAVICON_PATH, icoBuffer));

    // 3. Sync manifest.webmanifest
    try {
      const manifestRaw = await fs.readFile(MANIFEST_PATH, 'utf8');
      const manifest: WebManifest = JSON.parse(manifestRaw);

      manifest.icons = PNG_SIZES.map((size) => ({
        src: path.relative(
          path.dirname(MANIFEST_PATH),
          path.join(PNG_OUTPUT_DIR, `icon-${size}x${size}.png`),
        ),
        sizes: `${size}x${size}`,
        type: 'image/png',
        purpose: 'maskable any',
      }));

      await fs.writeFile(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n');
    } catch {
      console.warn('⚠️ manifest.webmanifest not found or unreadable, skipping manifest update.');
    }

    console.log('✓ Favicons and manifest icons generated successfully.');
  } catch (error) {
    console.error('✗ Error generating icons:', error);
    process.exit(1);
  }
}

interface ManifestIcon {
  src: string;
  sizes: string;
  type: string;
  purpose: string;
}

interface WebManifest {
  icons?: ManifestIcon[];
  [key: string]: unknown;
}

/**
 * Creates an ICO file from an array of PNG buffers.
 * @param pngBuffers
 * @returns A promise that resolves to a buffer containing the ICO file data.
 */
async function createIcoFromPngs(pngBuffers: Buffer[]): Promise<Buffer> {
  const numImages = pngBuffers.length;
  const headerSize = 6 + numImages * 16;

  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // Reserved
  header.writeUInt16LE(1, 2); // Image type (1 = ICO)
  header.writeUInt16LE(numImages, 4); // Number of images

  const entries: Buffer[] = [];
  let offset = headerSize;

  for (const png of pngBuffers) {
    const metadata = await sharp(png).metadata();
    const entry = Buffer.alloc(16);

    entry.writeUInt8(metadata.width! >= 256 ? 0 : metadata.width!, 0);
    entry.writeUInt8(metadata.height! >= 256 ? 0 : metadata.height!, 1);
    entry.writeUInt8(0, 2); // Color palette
    entry.writeUInt8(0, 3); // Reserved
    entry.writeUInt16LE(1, 4); // Color planes
    entry.writeUInt16LE(32, 6); // Bits per pixel
    entry.writeUInt32LE(png.length, 8); // Size of PNG data
    entry.writeUInt32LE(offset, 12); // Offset of PNG data

    entries.push(entry);
    offset += png.length;
  }

  return Buffer.concat([header, ...entries, ...pngBuffers]);
}

generateIcons();
