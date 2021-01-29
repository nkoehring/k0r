-- this script is meant to be used for benchmarking k0r with wrk
-- it generates different URL paths to simulate lots of different requests
-- run with `wrk -t8 -c400 -d30s -s random-path.lua http://127.0.0.1:8080/`

local fmt = string.format
local rnd = math.random

local alphabet = {"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"}

function init(args)
   local r = {}

   for i = 1, 42 do
     r[i] = wrk.format(nil, fmt("/%d%s", rnd(9), alphabet[rnd(#alphabet)]))
   end

   req = table.concat(r)
end

function request()
   return req
end
