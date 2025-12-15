import { render } from "@opentui/solid";
import { App } from "./App.js";
import type { TUIOptions } from "./types/index.js";
import { restoreTerminalState } from "./utils/cleanup.js";

export type { TUIOptions };

export async function launchTUI(options?: TUIOptions) {
  process.on('uncaughtException', (error) => {
    restoreTerminalState();
    console.error('Uncaught exception:', error);
    process.exit(1);
  });
  
  process.on('unhandledRejection', (reason) => {
    restoreTerminalState();
    console.error('Unhandled rejection:', reason);
    process.exit(1);
  });

  await render(() => <App {...(options ?? {})} />, {
    exitOnCtrlC: false,
    useAlternateScreen: true,
    useMouse: true,
    targetFps: 60,
    useKittyKeyboard: {},
  } as any);
}
