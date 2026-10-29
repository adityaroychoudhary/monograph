import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  LineChart, Line, XAxis, YAxis, Tooltip, CartesianGrid, ResponsiveContainer,
} from "recharts";

interface Bucket {
  t: number;
  work: number;
  personal: number;
}

export default function UsageChart() {
  const [data, setData] = useState<{ time: string; work: number; personal: number }[]>([]);

  useEffect(() => {
    const load = () => {
      const to = Date.now();
      const from = to - 24 * 3600 * 1000;
      invoke<Bucket[]>("usage_buckets", { fromMs: from, toMs: to, bucketMin: 30 }).then((b) =>
        setData(
          b.map((x) => ({
            time: new Date(x.t).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
            work: Math.round(x.work / 600) / 100,     // ms -> minutes, 2 decimals
            personal: Math.round(x.personal / 600) / 100,
          }))
        )
      );
    };
    load();
    const t = setInterval(load, 30000);
    return () => clearInterval(t);
  }, []);

  return (
    <ResponsiveContainer width="100%" height={220}>
      <LineChart data={data}>
        <CartesianGrid strokeDasharray="3 3" opacity={0.2} />
        <XAxis dataKey="time" interval="preserveStartEnd" />
        <YAxis unit="m" domain={[0, 30]} />
        <Tooltip />
        <Line type="stepAfter" dataKey="work" stroke="#3b82f6" strokeWidth={2}
              dot={false} isAnimationActive={false} />
        <Line type="stepAfter" dataKey="personal" stroke="currentColor" strokeWidth={2}
              strokeDasharray="5 4" dot={false} isAnimationActive={false} />
      </LineChart>
    </ResponsiveContainer>
  );
}