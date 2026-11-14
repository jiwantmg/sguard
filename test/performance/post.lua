-- Reads payload.json next to this script and sends it with a Bearer token
local function script_dir()
  local src = debug.getinfo(1, "S").source:sub(2)
  return src:match("(.*/)") or "./"
end

local function read_file(path)
  local f = assert(io.open(path, "r"), "cannot open " .. path)
  local content = f:read("*a")
  f:close()
  return content
end

local token = os.getenv("TOKEN")
assert(token and token ~= "", "TOKEN env var is required")

wrk.method = "POST"
wrk.body   = read_file(script_dir() .. "payload.json")
wrk.headers["Content-Type"]  = "application/json"
wrk.headers["Authorization"] = "Bearer " .. token

-- status code counters (per thread)
ok, unauthorized, other = 0, 0, 0
local threads = {}

function setup(thread)
  table.insert(threads, thread)
end

function response(status)
  if status == 200 then ok = ok + 1
  elseif status == 401 then unauthorized = unauthorized + 1
  else other = other + 1 end
end

function done(summary, latency, requests)
  local a, b, c = 0, 0, 0
  for _, t in ipairs(threads) do
    a = a + t:get("ok"); b = b + t:get("unauthorized"); c = c + t:get("other")
  end
  io.write(string.format("\n200: %d | 401: %d | other: %d\n", a, b, c))
end