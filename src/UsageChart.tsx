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

const HOUR = 3600 * 1000;

const RANGES = {
  day:   { label: "Day",   span: 24 * HOUR,       bucketMin: 30,   unit: "m", max: 30 },
  week:  { label: "Week",  span: 7 * 24 * HOUR,   bucketMin: 180,  unit: "h", max: 3 },
  month: { label: "Month", span: 30 * 24 * HOUR,  bucketMin: 1440, unit: "h", max: 24 },
  year:  { label: "Year",  span: 365 * 24 * HOUR, bucketMin: 1440, unit: "h", max: 24 },
} as const;

type RangeKey = keyof typeof RANGES;

const tickLabel = (key: RangeKey, t: number) => {
  const d = new Date(t);
  if (key === "day") return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  if (key === "week")
    return d.toLocaleString([], { weekday: "short", hour: "2-digit", minute: "2-digit" });
  return d.toLocaleDateString([], { day: "numeric", month: "short" });
};

const fmtDate = (ms: number) =>
  new Date(ms).toLocaleDateString([], { day: "numeric", month: "short", year: "numeric" });

export default function UsageChart() {
  const [range, setRange] = useState<RangeKey>("day");
  const [back, setBack] = useState(0); // how many periods back from now
  const [title, setTitle] = useState("");
  const [data, setData] = useState<{ label: string; work: number; personal: number }[]>([]);

  const cfg = RANGES[range];

  useEffect(() => {
    const load = () => {
      const to = Date.now() - back * cfg.span;
      const from = to - cfg.span;
      const div = cfg.unit === "m" ? 60000 : HOUR;
      invoke<Bucket[]>("usage_buckets", {
        fromMs: from,
        toMs: to,
        bucketMin: cfg.bucketMin,
        tzOffsetMin: -new Date().getTimezoneOffset(),
      }).then((b) => {
        setData(
          b.map((x) => ({
            label: tickLabel(range, x.t),
            work: Math.round((x.work / div) * 100) / 100,
            personal: Math.round((x.personal / div) * 100) / 100,
          }))
        );
        setTitle(`${fmtDate(from)} – ${fmtDate(to)}`);
      });
    };
    load();
    // only auto-refresh when looking at the present
    const timer = back === 0 ? setInterval(load, 30000) : undefined;
    return () => {
      if (timer) clearInterval(timer);
    };
  }, [range, back]);

  return (
    <>
      <div className="chart-controls">
        <div>
          {(Object.keys(RANGES) as RangeKey[]).map((k) => (
            <button
              key={k}
              className={range === k ? "active" : ""}
              onClick={() => {
                setRange(k);
                setBack(0);
              }}
            >
              {RANGES[k].label}
            </button>
          ))}
        </div>
        <div>
          <button onClick={() => setBack(back + 1)}>←</button>
          <span>{title}</span>
          <button disabled={back === 0} onClick={() => setBack(back - 1)}>→</button>
        </div>
      </div>

      <ResponsiveContainer width="100%" height={220}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" opacity={0.2} />
          <XAxis dataKey="label" interval="preserveStartEnd" />
          <YAxis unit={cfg.unit} domain={[0, cfg.max]} />
          <Tooltip />
          <Line type="stepAfter" dataKey="work" stroke="#3b82f6" strokeWidth={2}
                dot={false} isAnimationActive={false} />
          <Line type="stepAfter" dataKey="personal" stroke="currentColor" strokeWidth={2}
                strokeDasharray="5 4" dot={false} isAnimationActive={false} />
        </LineChart>
      </ResponsiveContainer>
    </>
  );
}