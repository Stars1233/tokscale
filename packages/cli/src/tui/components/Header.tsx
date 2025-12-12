import { Show } from "solid-js";
import type { TabType } from "../types/index.js";

interface HeaderProps {
  activeTab: TabType;
  onTabClick?: (tab: TabType) => void;
}

export function Header(props: HeaderProps) {
  return (
    <box flexDirection="row" paddingX={1} paddingY={0} justifyContent="space-between">
      <box flexDirection="row" gap={2}>
        <Tab name="Overview" tabId="overview" active={props.activeTab === "overview"} onClick={props.onTabClick} />
        <Tab name="Models" tabId="model" active={props.activeTab === "model"} onClick={props.onTabClick} />
        <Tab name="Daily" tabId="daily" active={props.activeTab === "daily"} onClick={props.onTabClick} />
        <Tab name="Stats" tabId="stats" active={props.activeTab === "stats"} onClick={props.onTabClick} />
      </box>
      <text fg="cyan" bold>Token Usage Tracker</text>
    </box>
  );
}

interface TabProps {
  name: string;
  tabId: TabType;
  active: boolean;
  onClick?: (tab: TabType) => void;
}

function Tab(props: TabProps) {
  const handleClick = () => props.onClick?.(props.tabId);

  return (
    <Show
      when={props.active}
      fallback={
        <box onMouseDown={handleClick}>
          <text dim>{props.name}</text>
        </box>
      }
    >
      <box onMouseDown={handleClick}>
        <text backgroundColor="cyan" fg="black" bold>{` ${props.name} `}</text>
      </box>
    </Show>
  );
}
