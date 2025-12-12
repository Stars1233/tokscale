import type { TabType } from "../App.js";

interface HeaderProps {
  activeTab: TabType;
}

export function Header({ activeTab }: HeaderProps) {
  return (
    <box paddingX={1} paddingY={0} justifyContent="space-between">
      <box gap={2}>
        <Tab name="Overview" active={activeTab === "overview"} />
        <Tab name="Models" active={activeTab === "model"} />
        <Tab name="Daily" active={activeTab === "daily"} />
        <Tab name="Stats" active={activeTab === "stats"} />
      </box>
      <text fg="cyan" bold>Token Usage Tracker</text>
    </box>
  );
}

function Tab({ name, active }: { name: string; active: boolean }) {
  if (active) {
    return (
      <box>
        <text backgroundColor="cyan" fg="black" bold>{` ${name} `}</text>
      </box>
    );
  }
  return <text dim>{name}</text>;
}
