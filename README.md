# sleepy

This simple proxy creates a HTTP-based rate limit via an injected `proxy-session` token, then applies rate limit based on configurable dynamic leaky bucket algorithm.

The rate limit is applied based on how long your server responds to a request. At a default configuration, it will *sleep* the request twice of TTFB, then as long as the identified request is not *greedy*, it will be as just fast.

Greedy in this term means the bucket didn't leak under the leaky bucket algorithm.

This proxy shouldn't punish your regular visitors -- it's only punish crawlers who:

- Doesn't keep the cookie
- Visit things too fast
- Frequently hit heavy endpoints

Configurations by envars:

```ini
REDIS=redis:// # optional redis database, if not set will use in mem
PORT=1238 # what port to bind, default is 8000

# accept float, by default the server response time will be multiplied by this
PENALTY_MULTIPLIER=3 

# all rate limit units below in miliseconds
LIMIT_CAP=20000 # how much computation time is allowed before penalty kicks in
LIMIT_INITIAL=20000 # initial value for new visitors, usually you want this the same as limit cap
LIMIT_IDLE_RATE=1 # how much value per milisecond the limit rate is go down by no traffic
LIMIT_SPEND_RATE=3 # how much value per milisecond the limit rate is go up by server spending
```

## Usage

```sh
go build -o sleepy
mv sleepy /usr/bin
sleepy npm start 
```
