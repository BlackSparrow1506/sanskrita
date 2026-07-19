#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ॐ  संस्कृता क्रीडाङ्गणम् (Sanskrita Playground)
Run संस्कृता code in your browser — with perfect Devanagari rendering.

Usage:
    python3 playground.py
then open  http://localhost:8765  (opens automatically).
Stop with Ctrl-C.
"""

import io
import json
import os
import signal
import sys
import urllib.parse
import webbrowser
from contextlib import redirect_stdout
from http.server import HTTPServer, BaseHTTPRequestHandler

import sanskrita

PORT = 8765
EX_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "examples")

PAGE = """<!doctype html>
<html lang="sa">
<head>
<meta charset="utf-8">
<title>ॐ संस्कृता क्रीडाङ्गणम्</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
  :root { --bg:#1e1b18; --panel:#2a2622; --accent:#e8a33d; --text:#f3ede4; --dim:#a99f90; }
  * { box-sizing: border-box; }
  body { margin:0; background:var(--bg); color:var(--text);
         font-family:"Noto Sans Devanagari","Devanagari MT",system-ui,sans-serif; }
  header { padding:14px 22px; border-bottom:2px solid var(--accent);
           display:flex; align-items:baseline; gap:14px; flex-wrap:wrap; }
  header h1 { margin:0; font-size:1.5rem; color:var(--accent); font-weight:700; }
  header span { color:var(--dim); font-size:.9rem; }
  main { display:grid; grid-template-columns:1fr 1fr; gap:16px; padding:16px 22px; }
  @media (max-width:800px){ main { grid-template-columns:1fr; } }
  .panel { background:var(--panel); border-radius:10px; padding:14px; }
  .panel h2 { margin:0 0 10px; font-size:1rem; color:var(--accent); font-weight:600; }
  textarea { width:100%; height:340px; background:#171412; color:var(--text);
             border:1px solid #47403a; border-radius:8px; padding:12px;
             font-family:"Noto Sans Devanagari","Devanagari MT",Menlo,monospace;
             font-size:1.15rem; line-height:1.9; resize:vertical; }
  textarea:focus { outline:2px solid var(--accent); }
  #out { min-height:340px; white-space:pre-wrap; background:#171412; border-radius:8px;
         padding:12px; font-size:1.25rem; line-height:1.9; }
  #out.err { color:#ff9d80; }
  .bar { display:flex; gap:10px; margin-top:12px; flex-wrap:wrap; }
  button { background:var(--accent); color:#1e1b18; border:0; border-radius:8px;
           padding:10px 22px; font-size:1.05rem; font-weight:700; cursor:pointer;
           font-family:inherit; }
  button:hover { filter:brightness(1.1); }
  button.ghost { background:transparent; color:var(--accent); border:1px solid var(--accent); }
  .hint { color:var(--dim); font-size:.85rem; margin-top:8px; }
</style>
</head>
<body>
<header>
  <h1>ॐ संस्कृता क्रीडाङ्गणम्</h1>
  <span>Sanskrita Playground — v0.1 अङ्कुरः</span>
</header>
<main>
  <div class="panel">
    <h2>लेखनम् (code)</h2>
    <textarea id="code" spellcheck="false"># अत्र लिखतु — write here
वद("नमस्ते जगत्")।

मानय क = १।
यावत् (क &lt;= ५) {
    वद("गणना:", क)।
    क = क + १।
}

वद("०.१ + ०.२ =", ०.१ + ०.२)।</textarea>
    <div class="bar">
      <button onclick="run()">▶ चालय (Run)</button>
      <button class="ghost" onclick="convert()">a→अ देवनागरी</button>
      <label style="display:flex;align-items:center;gap:6px;color:var(--dim);font-size:.9rem">
        <input type="checkbox" id="auto" checked> स्वयम् (auto)
      </label>
    </div>
    <div class="bar" id="exbar"></div>
    <h2 style="margin-top:14px">प्रवेशः (answers for पृच्छ — one per line)</h2>
    <textarea id="stdin" style="height:70px" spellcheck="false"
      placeholder="यदि कार्यक्रमः पृच्छति, उत्तराणि अत्र लिखतु — if the program asks, type answers here first"></textarea>
    <div class="hint">Ctrl/Cmd + Enter = चालय । keywords: मानय ध्रुव वद यदि अथ अन्यथा यावत् च वा न — or roman: manay vada yadi…</div>
  </div>
  <div class="panel">
    <h2>फलम् (output)</h2>
    <div id="out">— अत्र फलं दृश्यते —</div>
  </div>
</main>
<script>
async function listEx(){
  try{
    const names = await (await fetch('/exlist')).json();
    const bar = document.getElementById('exbar');
    bar.innerHTML = '';
    for(const n of names){
      const b = document.createElement('button');
      b.className = 'ghost';
      b.textContent = n.replace(/\\.(सं|sam)$/,'');
      b.onclick = ()=>loadEx(n);
      bar.appendChild(b);
    }
  }catch(e){}
}
listEx();
async function loadEx(name){
  const r = await fetch('/ex/' + encodeURIComponent(name));
  if(!r.ok){ document.getElementById('out').textContent = 'सञ्चिका न प्राप्ता / file not found: ' + name; return; }
  document.getElementById('code').value = await r.text();
  run();
}
async function run(){
  const out = document.getElementById('out');
  out.className=''; out.textContent = '…';
  try{
    const r = await fetch('/run', {method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({code: document.getElementById('code').value,
                            stdin: document.getElementById('stdin').value})});
    const d = await r.json();
    out.textContent = d.output || '(शून्यं फलम् — no output)';
    if(!d.ok) out.className='err';
  }catch(e){ out.textContent='सेवकदोषः / server error: '+e; out.className='err'; }
}
async function convert(){
  const ta = document.getElementById('code');
  const pos = ta.selectionStart;
  const r = await fetch('/convert', {method:'POST',
    headers:{'Content-Type':'application/json'},
    body: JSON.stringify({code: ta.value, prefix: ta.value.slice(0, pos)})});
  const d = await r.json();
  if(d.code === ta.value) return;
  ta.value = d.code;
  ta.setSelectionRange(d.prefixLen, d.prefixLen);
  ta.focus();
}
document.getElementById('code').addEventListener('keydown', e=>{
  if((e.ctrlKey||e.metaKey) && e.key==='Enter'){ e.preventDefault(); run(); }
});
document.getElementById('code').addEventListener('input', e=>{
  if(!document.getElementById('auto').checked) return;
  if(e.data && ' |(){}।\\n'.includes(e.data)) convert();
});
document.getElementById('code').addEventListener('keyup', e=>{
  if(document.getElementById('auto').checked && e.key==='Enter') convert();
});
</script>
</body>
</html>
"""


class _Timeout(Exception):
    pass


def _alarm(signum, frame):
    raise _Timeout()


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):          # keep the console quiet
        pass

    def do_GET(self):
        if self.path == "/exlist":                # list example files
            try:
                names = sorted(n for n in os.listdir(EX_DIR)
                               if n.endswith((".सं", ".sam")))
            except OSError:
                names = []
            body = json.dumps(names).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path.startswith("/ex/"):          # serve real example files, live
            name = urllib.parse.unquote(self.path[4:])
            if "/" in name or "\\" in name or ".." in name:
                self.send_response(400); self.end_headers(); return
            try:
                with open(os.path.join(EX_DIR, name), encoding="utf-8") as f:
                    body = f.read().encode("utf-8")
            except OSError:
                self.send_response(404); self.end_headers(); return
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.end_headers()
        self.wfile.write(PAGE.encode("utf-8"))

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        try:
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            code = payload.get("code", "")
        except Exception:
            payload, code = {}, ""
        if self.path == "/convert":
            body = json.dumps({
                "code": sanskrita.devanagarify(code),
                "prefixLen": len(sanskrita.devanagarify(payload.get("prefix", ""))),
            }).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path != "/run":
            self.send_response(404)
            self.end_headers()
            return
        buf = io.StringIO()
        ok = True
        interp = sanskrita.Interpreter()
        old_stdin = sys.stdin
        sys.stdin = io.StringIO(payload.get("stdin", ""))   # feeds पृच्छ
        signal.signal(signal.SIGALRM, _alarm)
        signal.alarm(5)                        # infinite-loop guard
        try:
            with redirect_stdout(buf):
                sanskrita.run_source(code, interp)
        except sanskrita.SanskritaError as err:
            ok = False
            buf.write(("\n" if buf.getvalue() else "") + str(err))
        except _Timeout:
            ok = False
            buf.write("\nकालातिक्रमः — ५ क्षणाः अतीताः (अनन्तं यावत्-चक्रम्?)\n"
                      "Timeout — ran over 5 seconds (infinite यावत् loop?)")
        except RecursionError:
            ok = False
            buf.write("\nअतिगभीरम् / too deeply nested")
        finally:
            signal.alarm(0)
            sys.stdin = old_stdin
        body = json.dumps({"ok": ok, "output": buf.getvalue()}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main():
    server = HTTPServer(("127.0.0.1", PORT), Handler)
    url = f"http://localhost:{PORT}"
    print("ॐ  संस्कृता क्रीडाङ्गणम् चलति —", url)
    print("   (playground running — open the link; Ctrl-C to stop)")
    try:
        webbrowser.open(url)
    except Exception:
        pass
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nपुनर्मिलामः ।")


if __name__ == "__main__":
    main()
