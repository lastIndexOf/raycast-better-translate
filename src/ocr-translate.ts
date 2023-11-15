import { open } from "@raycast/api";
import { readImageText, readOcrImage } from "./helper";

export default async function main() {
  const ocrImage = await readOcrImage();
  const word = await readImageText(ocrImage);

  await open(`raycast://extensions/raycast/translator/translate?fallbackText=${word}`);
}
