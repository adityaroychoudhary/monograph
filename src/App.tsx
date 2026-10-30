import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import UsageChart from "./UsageChart";

type Group = "personal" | "work" | "unassigned";
interface AppRow {
  name: string;
  grp: Group;
  tracked: boolean,
  total_secs: number;
}

const fmt = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h ? `${h}h ${m}m` : `${m}m ${s % 60}s`;
};

export default function App() {
  const [apps, setApps] = useState<AppRow[]>([]);
  const [autostart, setAutostart] = useState(false);

  const load = () => invoke<AppRow[]>("list_apps").then(setApps);

  useEffect(() => {
    load();
    const t = setInterval(load, 5000);
    return () => clearInterval(t);
  }, []);
  
  useEffect(() => {
  invoke<boolean>("get_autostart").then(setAutostart);
}, []);

  const assign = async (name: string, grp: Group) => {
    await invoke("set_group", { name, grp });
    load();
  };
  
  const toggleTracked = async (name: string, tracked: boolean) => {
  await invoke("set_tracked", { name, tracked });
  load();
};

const toggleAutostart = async (enabled: boolean) => {
  await invoke("set_autostart", { enabled });
  setAutostart(enabled);
};

  return (
    <main>
    <UsageChart/>
      <h1>Apps</h1>
      <table>
        <thead>
          <tr><th>Track</th><th>App</th><th>Total</th><th>Group</th></tr>
        </thead>
        <tbody>
    {apps.map((a) => (
      <tr key={a.name}>
        <td>
          <input
            type="checkbox"
            checked={a.tracked}
            onChange={(e) => toggleTracked(a.name, e.target.checked)}
          />
        </td>
        <td>{a.name}</td>
        <td>{fmt(a.total_secs)}</td>
        <td>
          {(["personal", "work"] as const).map((g) => (
            <button
              key={g}
              className={a.grp === g ? "active" : ""}
              onClick={() => assign(a.name, a.grp === g ? "unassigned" : g)}
            >
              {g}
            </button>
          ))}
        </td>
      </tr>
    ))}
  </tbody>
      </table>
      <label className="autostart">
  <input
    type="checkbox"
    checked={autostart}
    onChange={(e) => toggleAutostart(e.target.checked)}
  />
  Start Monograph at login
</label>
    </main>
  );
}