import { open } from "@raycast/api";
import { readSelectionWord } from "./helper";

export default async function main() {
  const word = await readSelectionWord();
  await open(`raycast://extensions/raycast/translator/translate?fallbackText=${word}`);
}
