/**
 * Terminal cleanup utility for restoring terminal state when TUI exits.
 * Provides fallback cleanup for crash scenarios where OpenTUI's destroy() may not run.
 */

/**
 * Complete terminal state restoration sequences.
 * Based on research from xterm, kitty keyboard protocol, and popular TUI libraries.
 */
export const TERMINAL_CLEANUP_SEQUENCES = [
  // Disable all mouse tracking modes
  '\x1b[?1016l',  // SGR Pixel Mode
  '\x1b[?1015l',  // URXVT Mouse
  '\x1b[?1006l',  // SGR Mouse (produces "51;77;17M" sequences)
  '\x1b[?1005l',  // UTF-8 Mouse
  '\x1b[?1004l',  // Focus Events
  '\x1b[?1003l',  // Any Event Mouse (motion tracking)
  '\x1b[?1002l',  // Button Event Mouse (drag tracking)
  '\x1b[?1001l',  // Highlight Mouse
  '\x1b[?1000l',  // VT200 Mouse
  '\x1b[?9l',     // X10 Mouse
  
  // Disable kitty keyboard protocol (produces "9;5u" sequences)
  '\x1b[<u',      // Disable kitty keyboard progressive enhancement
  '\x1b[>4;0m',   // Disable modifyOtherKeys (xterm)
  
  // Disable synchronized updates
  '\x1b[?2026l',
  
  // Restore cursor and attributes
  '\x1b[?25h',    // Show cursor (DECTCEM)
  '\x1b[0m',      // Reset all text attributes (SGR 0)
  
  // Exit alternate screen buffer
  '\x1b[?1049l',  // Exit alt screen + restore cursor
  '\x1b[?47l',    // Legacy: exit alt screen
  '\x1b[?1047l',  // Legacy: another alt screen mode
].join('');

let hasCleanedUp = false;

/**
 * Write terminal restoration sequences to stdout.
 * Idempotent - safe to call multiple times.
 * 
 * This is a FALLBACK for crash scenarios. Normal exit should use
 * renderer.destroy() which handles cleanup properly.
 */
export function restoreTerminalState(): void {
  if (hasCleanedUp) return;
  hasCleanedUp = true;
  
  try {
    process.stdout.write(TERMINAL_CLEANUP_SEQUENCES);
  } catch {
    // Ignore write errors (stdout may be closed)
  }
}

/**
 * Reset cleanup state (for testing purposes).
 */
export function resetCleanupState(): void {
  hasCleanedUp = false;
}
