import { Clipboard } from "@raycast/api";
import { runAppleScript } from "@raycast/utils";

export const readSelectionWord = async () => {
  const originalContent = await Clipboard.read();

  const script = `
tell application "System Events"
  keystroke "c" using command down
end tell

delay 0.05
`;

  await runAppleScript(script);
  const selectionWord = await Clipboard.readText();

  await Clipboard.copy(originalContent);

  return selectionWord;
};
