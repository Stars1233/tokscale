import { createSignal, onMount, onCleanup } from "solid-js";

const COLORS = ["#00FFFF", "#00D7D7", "#00AFAF", "#008787", "#666666", "#666666", "#666666", "#666666"];
const WIDTH = 8;
const HOLD_START = 30;
const HOLD_END = 9;
const TRAIL_LENGTH = 4;
const INTERVAL = 40;

interface SpinnerState {
  position: number;
  forward: boolean;
}

function getScannerState(frame: number): SpinnerState {
  const forwardFrames = WIDTH;
  const backwardFrames = WIDTH - 1;
  const totalCycle = forwardFrames + HOLD_END + backwardFrames + HOLD_START;
  const normalized = frame % totalCycle;

  if (normalized < forwardFrames) {
    return { position: normalized, forward: true };
  } else if (normalized < forwardFrames + HOLD_END) {
    return { position: WIDTH - 1, forward: true };
  } else if (normalized < forwardFrames + HOLD_END + backwardFrames) {
    return { position: WIDTH - 2 - (normalized - forwardFrames - HOLD_END), forward: false };
  }
  return { position: 0, forward: false };
}

interface LoadingSpinnerProps {
  message?: string;
}

export function LoadingSpinner(props: LoadingSpinnerProps) {
  const [frame, setFrame] = createSignal(0);

  onMount(() => {
    const id = setInterval(() => {
      setFrame((f) => f + 1);
    }, INTERVAL);
    onCleanup(() => clearInterval(id));
  });

  const state = () => getScannerState(frame());

  const getCharProps = (index: number) => {
    const { position, forward } = state();
    const distance = forward ? position - index : index - position;

    if (distance >= 0 && distance < TRAIL_LENGTH) {
      return { char: "■", color: COLORS[distance] };
    }
    return { char: "⬝", color: "#444444" };
  };

  return (
    <box flexDirection="column" justifyContent="center" alignItems="center" flexGrow={1}>
      <box flexDirection="row" gap={0}>
        {Array.from({ length: WIDTH }, (_, i) => {
          const { char, color } = getCharProps(i);
          return <text fg={color}>{char}</text>;
        })}
      </box>
      <box marginTop={1}>
        <text dim>{props.message || "Loading data..."}</text>
      </box>
    </box>
  );
}
