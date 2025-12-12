declare module "@opentui/core" {
  export interface CliRendererConfig {
    exitOnCtrlC?: boolean;
    targetFps?: number;
    backgroundColor?: string;
    useAlternateScreen?: boolean;
    useMouse?: boolean;
  }

  export interface CliRenderer {
    root: {
      add: (renderable: unknown) => void;
    };
    start: () => void;
    stop: () => void;
    console: {
      show: () => void;
    };
  }

  export function createCliRenderer(config?: CliRendererConfig): Promise<CliRenderer>;
  
  export interface KeyEvent {
    name: string;
    eventType: "press" | "release";
    repeated?: boolean;
  }
}

declare module "@opentui/react" {
  import type { ReactNode } from "react";
  import type { CliRenderer, KeyEvent } from "@opentui/core";

  export interface Root {
    render: (node: ReactNode) => void;
    unmount: () => void;
  }

  export function createRoot(renderer: CliRenderer): Root;
  
  export function useKeyboard(
    handler: (key: KeyEvent) => void,
    options?: { release?: boolean }
  ): void;
  
  export function useTerminalDimensions(): { width: number; height: number };
  
  export function useRenderer(): CliRenderer;
  
  export function useOnResize(callback: (width: number, height: number) => void): void;
}
