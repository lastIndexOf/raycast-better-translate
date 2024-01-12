import { getSelectedText, open } from "@raycast/api";

export default async function main() {
  const word = await getSelectedText();
  await open(`raycast://extensions/raycast/translator/translate?fallbackText=${word}`);
}
