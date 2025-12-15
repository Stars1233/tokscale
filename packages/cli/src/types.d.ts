declare module "string-width" {
  export default function stringWidth(str: string): number;
}

declare module "bun" {
  export interface BunSubprocess {
    stdin: any;
    stdout: any;
    stderr: any;
    exited: Promise<number>;
    kill(signal?: string): void;
  }

  export function spawn(options: {
    cmd: string[];
    stdin?: "pipe";
    stdout?: "pipe";
    stderr?: "pipe";
    timeout?: number;
    killSignal?: string;
  }): BunSubprocess;
}
