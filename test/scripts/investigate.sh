#!/usr/bin/env bash
# Root-cause investigation for gateway connection errors under load.
# Usage:  GW_PID=<pid> ./investigate.sh [seconds=30] [connections=1000]
# Run it right after a FRESH gateway start (cold pool) and after TIME_WAIT has drained.
set -u

DUR=${1:-30}
CONNS=${2:-1000}
GW_PORT=${GW_PORT:-8080}
UP_PORT=${UP_PORT:-9000}
URL=${URL:-http://127.0.0.1:${GW_PORT}/health}
COOKIE=${COOKIE:-session=valid-session-id}
NGINX_CT=${NGINX_CT:-bench-upstream}
GW_PID=${GW_PID:-$(pgrep -f 'target/release/sguard' | head -1)}
OUT=investigate-$(date +%H%M%S)
mkdir -p "$OUT"

[ -n "$GW_PID" ] && [ -d "/proc/$GW_PID" ] || { echo "set GW_PID=<gateway pid>"; exit 1; }

# ---------- 1. static facts ----------
{
  echo "== gateway process"
  echo "pid=$GW_PID  fds_now=$(ls /proc/$GW_PID/fd | wc -l)"
  grep -E "open files" /proc/$GW_PID/limits
  echo "== this shell (wrk) ulimit -n: $(ulimit -n)"
  echo "== kernel"
  sysctl net.core.somaxconn net.ipv4.tcp_max_syn_backlog net.ipv4.ip_local_port_range \
         net.ipv4.tcp_tw_reuse net.ipv4.tcp_fin_timeout 2>/dev/null
  echo "== listeners (Send-Q = effective accept backlog)"
  ss -ltn "( sport = :$GW_PORT or sport = :$UP_PORT )"
  echo "== nginx effective config"
  docker exec "$NGINX_CT" nginx -T 2>/dev/null \
    | grep -E "worker_processes|worker_connections|keepalive_requests|keepalive_timeout|listen " \
    || echo "(could not read nginx config: set NGINX_CT=<container name>)"
  echo "TIME_WAIT before run: $(ss -tanH state time-wait | wc -l)"
} > "$OUT/static.txt"

nstat -az 2>/dev/null | awk '!/^#/ {print $1, $2}' | sort > "$OUT/nstat.before"

# ---------- 2. load + per-second sampling ----------
wrk -t5 -c"$CONNS" -d"${DUR}s" --latency -H "Cookie: $COOKIE" "$URL" > "$OUT/wrk.txt" 2>&1 &
WRK=$!

# Who closed first? The side that closes first holds the TIME_WAIT socket.
classify_tw() {
  ss -tanH state time-wait | awk -v up="$UP_PORT" -v gw="$GW_PORT" '
    { n=split($3,l,":"); lp=l[n]; n=split($4,p,":"); pp=p[n]; t++
      if (lp==up) a++; else if (pp==up) b++; else if (lp==gw) c++; else if (pp==gw) d++; else e++ }
    END { printf "%d,%d,%d,%d,%d,%d", t, a, b, c, d, e }'
}

echo "sec,tw_total,tw_nginx_closed(sport=$UP_PORT),tw_gateway_closed_upstream(dport=$UP_PORT),tw_gateway_closed_client(sport=$GW_PORT),tw_client_closed(dport=$GW_PORT),tw_other,est_to_upstream,est_from_clients,gw_fds,upstream_acceptq,syn_recv" > "$OUT/samples.csv"
s=0
while kill -0 "$WRK" 2>/dev/null; do
  s=$((s+1))
  est_up=$(ss -tanH state established "( dport = :$UP_PORT )" | wc -l)
  est_cl=$(ss -tanH state established "( sport = :$GW_PORT )" | wc -l)
  fds=$(ls /proc/$GW_PID/fd 2>/dev/null | wc -l)
  aq=$(ss -ltnH "( sport = :$UP_PORT )" | awk '{print $2; exit}')
  syn=$(ss -tanH state syn-recv | wc -l)
  echo "$s,$(classify_tw),$est_up,$est_cl,$fds,${aq:-0},$syn" >> "$OUT/samples.csv"
  sleep 1
done
wait "$WRK" 2>/dev/null

# ---------- 3. kernel counters that changed during the run ----------
nstat -az 2>/dev/null | awk '!/^#/ {print $1, $2}' | sort > "$OUT/nstat.after"
join "$OUT/nstat.before" "$OUT/nstat.after" \
  | awk '$3-$2>0 {print $1, $3-$2}' \
  | grep -E "Listen|ReqQ|TimeWait|TW|AttemptFails|EstabResets|SynRetrans|Retrans|Abort|Overflow|Drop" \
  > "$OUT/nstat.delta"

# ---------- 4. verdict ----------
awk -F, -v up="$UP_PORT" -v gw="$GW_PORT" '
  NR>1 { for (i=2;i<=12;i++) if ($i>mx[i]) mx[i]=$i }
  END {
    t=mx[2]
    printf "\nPEAKS: time_wait=%d  est_to_upstream=%d  est_from_clients=%d  gateway_fds=%d  upstream_acceptq=%d  syn_recv=%d\n", t, mx[8], mx[9], mx[10], mx[11], mx[12]
    printf "TIME_WAIT by who closed first: nginx=%d  gateway->upstream=%d  gateway->client=%d  client/wrk=%d  other=%d\n\n", mx[3], mx[4], mx[5], mx[6], mx[7]
    if (t < 1000) print "* TIME_WAIT stayed low: connections are being reused (no churn)."
    if (t >= 1000 && mx[3] >= 0.5*t) print "* CAUSE A: nginx closes connections first -> nginx keepalive_requests/keepalive_timeout too low."
    if (t >= 1000 && mx[4] >= 0.5*t) print "* CAUSE B: gateway closes upstream connections -> pool too small / idle eviction / error paths discarding sockets."
    if (t >= 1000 && mx[5] >= 0.5*t) print "* CAUSE C: gateway closes CLIENT connections -> service returning Err, parse errors, or header_read_timeout. Check gateway log for connection error lines."
    if (t >= 1000 && mx[6] >= 0.5*t) print "* CAUSE D: wrk closes connections -> responses too slow (wrk timeout), usually upstream queueing."
    if (mx[8] > mx[9] && mx[8] > 600) print "* Pool opened more upstream connections than expected for the load -> no cap on concurrent upstream connections."
  }' "$OUT/samples.csv"

echo
echo "== kernel counters that increased (ListenOverflows/ListenDrops > 0 => accept queue overflow):"
cat "$OUT/nstat.delta"
echo
echo "== wrk:"; grep -E "Requests/sec|99%|Non-2xx|Socket errors" "$OUT/wrk.txt"
echo
echo "Static facts: $OUT/static.txt   Per-second samples: $OUT/samples.csv"
echo "Paste: $OUT/static.txt, the PEAKS/CAUSE lines above, the counters, and the gateway log summary lines."
