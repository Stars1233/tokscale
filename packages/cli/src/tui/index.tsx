import { render } from "@opentui/solid";
import { App } from "./App.js";
import type { TUIOptions } from "./types/index.js";

export type { TUIOptions };

export async function launchTUI(options?: TUIOptions) {
  await render(() => <App {...(options ?? {})} />, {
    exitOnCtrlC: false,
    useAlternateScreen: true,
    useMouse: true,
    targetFps: 60,
    useKittyKeyboard: {},
  } as any);
}
