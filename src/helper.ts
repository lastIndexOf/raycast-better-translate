import { Clipboard } from "@raycast/api";
import { runAppleScript } from "@raycast/utils";
// import { spawn } from "child_process";

export const readSelectionWord = async () => {
  const originalContent = await Clipboard.read();

  const script = `
tell application "System Events"
  set activeApp to name of first application process whose frontmost is true
  tell process activeApp
      keystroke "c" using command down
  end tell
end tell

delay 0.1

activeApp
`;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const output = await runAppleScript(script);
  const selectionWord = await Clipboard.readText();

  await Clipboard.copy(originalContent);

  console.info(output);
  return selectionWord;
};

// const runAppleScript = (cmd: string) => {
//   return new Promise((resolve, reject) => {
//     const ls = spawn("osascript", ["-e", cmd]);

//     let stdout = "";
//     let stderr = "";
//     ls.stdout.on("data", (data: Buffer) => {
//       stdout = data.toString();
//     });

//     ls.stderr.on("data", (data: Buffer) => {
//       stderr = data.toString();
//     });

//     ls.on("close", (code: number) => {
//       resolve({ code, stdout, stderr });
//     });

//     ls.on("error", (err) => {
//       reject(err);
//     });
//   });
// };
