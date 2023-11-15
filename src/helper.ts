import path from "path";
import { exec } from "child_process";
import { Clipboard, environment } from "@raycast/api";
import { runAppleScript } from "@raycast/utils";
import { chmod } from "fs/promises";

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

  return selectionWord ?? "";
};

export const readOcrImage = async () => {
  const ocr = path.resolve(environment.assetsPath, "ocr");
  await chmod(ocr, "755");
  await execCmd(ocr);

  const ocrImage = `/tmp/raycast-better-ocr-target.png`;
  return ocrImage;
};

export const readImageText = async (image: string) => {
  const script = `use framework "Vision"

  on getImageText(imagePath)
      -- Get image content
      set theImage to current application's NSImage's alloc()'s initWithContentsOfFile:imagePath
  
       -- Set up request handler using image's raw data
      set requestHandler to current application's VNImageRequestHandler's alloc()'s initWithData:(theImage's TIFFRepresentation()) options:(current application's NSDictionary's alloc()'s init())
      
      -- Initialize text request
      set theRequest to current application's VNRecognizeTextRequest's alloc()'s init()
    
       -- Perform the request and get the results
      requestHandler's performRequests:(current application's NSArray's arrayWithObject:(theRequest)) |error|:(missing value)
      set theResults to theRequest's results()
  
      -- Obtain and return the string values of the results
      set theText to {}
      repeat with observation in theResults
          copy ((first item in (observation's topCandidates:1))'s |string|() as text) to end of theText
      end repeat
      return theText
  end getImageText
  
  on run (argv)
      set imagePath to "${image}"
      getImageText(imagePath)
  end run`;

  const res = await runAppleScript(script);

  return res;
};

const execCmd = (cmd: string) =>
  new Promise((resolve, reject) => {
    exec(cmd, { env: { ...process.env } }, (error, stdout, stderr) => {
      if (error) {
        reject(error);
        return;
      }

      resolve({ stdout, stderr });
    });
  });
