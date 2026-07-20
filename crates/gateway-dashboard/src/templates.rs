//! Embedded HTML templates for the dashboard.
//!
//! The entire dashboard UI is a single self-contained HTML page with
//! inline CSS and minimal vanilla JS — no external dependencies.

pub fn dashboard_html(active_sessions: usize, uptime_secs: u64) -> String {
    let uptime = format_uptime(uptime_secs);
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>KSP Gateway Dashboard</title>
  <style>
    :root {{
      --bg:       #0d0e10;
      --surface:  #16181c;
      --border:   #252830;
      --text:     #e8eaf0;
      --muted:    #6b7280;
      --accent:   #f5c518;
      --success:  #22c55e;
      --font: 'Inter', system-ui, sans-serif;
      --mono: 'JetBrains Mono', 'Fira Code', monospace;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      display: flex;
    }}
    nav {{
      width: 200px;
      background: var(--surface);
      border-right: 1px solid var(--border);
      padding: 24px 0;
      flex-shrink: 0;
    }}
    nav .logo {{
      padding: 0 20px 24px;
      font-size: 16px;
      font-weight: 700;
      color: var(--accent);
      letter-spacing: 0.05em;
    }}
    nav .logo span {{ color: var(--muted); font-weight: 400; }}
    nav a {{
      display: block;
      padding: 10px 20px;
      color: var(--muted);
      text-decoration: none;
      font-size: 14px;
      transition: color 0.15s, background 0.15s;
    }}
    nav a:hover, nav a.active {{
      color: var(--text);
      background: rgba(245,197,24,0.07);
      border-right: 2px solid var(--accent);
    }}
    main {{
      flex: 1;
      padding: 32px;
      overflow-y: auto;
    }}
    h1 {{ font-size: 22px; font-weight: 600; margin-bottom: 6px; }}
    .subtitle {{ color: var(--muted); font-size: 14px; margin-bottom: 28px; }}
    .status-pill {{
      display: inline-flex;
      align-items: center;
      gap: 6px;
      background: rgba(34,197,94,0.12);
      color: var(--success);
      font-size: 12px;
      font-weight: 600;
      padding: 4px 10px;
      border-radius: 100px;
      margin-bottom: 28px;
    }}
    .status-pill::before {{
      content: '';
      width: 7px; height: 7px;
      border-radius: 50%;
      background: var(--success);
      animation: pulse 2s infinite;
    }}
    @keyframes pulse {{
      0%,100% {{ opacity: 1; }}
      50% {{ opacity: 0.4; }}
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: 16px;
      margin-bottom: 32px;
    }}
    .card {{
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 20px;
    }}
    .card .label {{
      font-size: 12px;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      margin-bottom: 8px;
    }}
    .card .value {{
      font-size: 28px;
      font-weight: 700;
      color: var(--accent);
      font-family: var(--mono);
    }}
    .card .sub {{
      font-size: 12px;
      color: var(--muted);
      margin-top: 4px;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }}
    th {{
      text-align: left;
      padding: 10px 12px;
      color: var(--muted);
      font-weight: 500;
      border-bottom: 1px solid var(--border);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }}
    td {{
      padding: 10px 12px;
      border-bottom: 1px solid rgba(37,40,48,0.5);
      font-family: var(--mono);
      color: var(--text);
    }}
    tr:last-child td {{ border-bottom: none; }}
    .section-title {{
      font-size: 14px;
      font-weight: 600;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      margin-bottom: 12px;
    }}
    footer {{
      margin-top: 40px;
      font-size: 12px;
      color: var(--muted);
    }}
  </style>
</head>
<body>
  <nav>
    <div class="logo">KSP <span>Gateway</span></div>
    <a href="/" class="active">Overview</a>
    <a href="/api/sessions">Sessions</a>
    <a href="/metrics">Metrics</a>
    <a href="/healthz">Health</a>
  </nav>
  <main>
    <div class="status-pill">RUNNING</div>
    <h1>Gateway Overview</h1>
    <p class="subtitle">KSP Gateway v{version} · Uptime {uptime}</p>

    <div class="grid">
      <div class="card">
        <div class="label">Active Sessions</div>
        <div class="value" id="sessions">{active_sessions}</div>
        <div class="sub">concurrent KSP connections</div>
      </div>
      <div class="card">
        <div class="label">Uptime</div>
        <div class="value" style="font-size:18px">{uptime}</div>
        <div class="sub">{uptime_secs}s total</div>
      </div>
      <div class="card">
        <div class="label">Listen Port</div>
        <div class="value">8765</div>
        <div class="sub">KSP protocol</div>
      </div>
      <div class="card">
        <div class="label">Dashboard</div>
        <div class="value">9090</div>
        <div class="sub">this interface</div>
      </div>
    </div>

    <div class="section-title">Active Sessions</div>
    <div class="card">
      <table>
        <thead>
          <tr>
            <th>Session ID</th>
            <th>Peer Address</th>
            <th>Connected</th>
            <th>Requests</th>
            <th>Bytes In</th>
            <th>Bytes Out</th>
          </tr>
        </thead>
        <tbody id="sessions-tbody">
          <tr><td colspan="6" style="color:var(--muted);text-align:center;padding:20px">
            No active sessions
          </td></tr>
        </tbody>
      </table>
    </div>

    <footer>KSP Gateway · <a href="/metrics" style="color:var(--accent)">Prometheus Metrics</a></footer>
  </main>

  <script>
    async function refresh() {{
      try {{
        const r = await fetch('/api/sessions');
        const sessions = await r.json();
        const tbody = document.getElementById('sessions-tbody');
        document.getElementById('sessions').textContent = sessions.length;
        if (sessions.length === 0) {{
          tbody.innerHTML = '<tr><td colspan="6" style="color:var(--muted);text-align:center;padding:20px">No active sessions</td></tr>';
          return;
        }}
        tbody.innerHTML = sessions.map(s => `
          <tr>
            <td>${{s.id}}</td>
            <td>${{s.peer}}</td>
            <td>${{new Date(s.connected_at).toLocaleTimeString()}}</td>
            <td>${{s.requests}}</td>
            <td>${{s.bytes_in}}</td>
            <td>${{s.bytes_out}}</td>
          </tr>
        `).join('');
      }} catch(e) {{
        console.error('Dashboard refresh error:', e);
      }}
    }}
    refresh();
    setInterval(refresh, 2000);
  </script>
</body>
</html>"##,
        version        = env!("CARGO_PKG_VERSION"),
        uptime         = uptime,
        uptime_secs    = uptime_secs,
        active_sessions = active_sessions,
    )
}

fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}
