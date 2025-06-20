# sleepy (WIP)

This simple proxy creates a HTTP-based rate limit via an injected `sleepy-session` token, then applies rate limit based on configurable dynamic leaky bucket algorithm.

The rate limit is applied based on how long your server responds to a request. At a default configuration, it will *sleep* the request twice of TTFB, then as long as the identified request is not *greedy*, it will be as just fast.

Greedy in this term means the bucket didn't leak under the leaky bucket algorithm.

This proxy shouldn't punish your regular visitors -- it's only punish crawlers who:

- Doesn't keep the cookie
- Visit things too fast
- Frequently hit heavy endpoints

Read [Design: Weight Scoring](#design-weight-scoring) for more info.

## Usage

```sh
cargo build --release
sudo mv target/release/sleepy /usr/local/bin
sleepy --upstream localhost:4000
```

## Configuration

Configurations by envars:

```ini
REDIS=redis:// # optional redis database, if not set will use in mem
PORT=1238 # what port to bind, default is 8000

# all rate limit units below in miliseconds
LIMIT_CAP=20000 # how much computation time is allowed before penalty kicks in
LIMIT_INITIAL=20000 # initial weight for new visitors, usually you want this the same as limit cap
LIMIT_IDLE_RATE=200 # how much weight per second the limit rate is go down by no traffic
LIMIT_SPEND_RATE=3000 # how much weight per second the limit rate is go up by server time

# if user weight > LIMIT_CAP, the excess sleep time is multiplied by this 
# only accept int
PENALTY_MULTIPLIER=1
# if user detected dropping keys while using same IP, and weight > LIMIT_CAP,
# how much additional last user weight is added to this new session?
# if you dislike this feature (to promote VPN/bcoz behind CDN/want less memory), set this to 0
# only accept int
SCRAPER_MULTIPLIER=0
# accept bool, whether the limit weight is emitted as `X-Sleepy-Weight`
EMIT_HEADERS=true


```

## Design: Weight Scoring

Actually, this software is a (\*my) response of recently viral [AI Bot Firewall](https://github.com/TecharoHQ/anubis), but some people (me too) might consider it a bit uneasy that to fight a bot we have to make our devices slightly a bit warm. And then i'm inspired with the way ethereum solve the energy dilemma against cryptocurrency system by embrancing Proof of Stake rather than Proof of Work.

In this concept, the proof of stake is in the cookies: As long as the cookie bearer is proven humane, the server bandwidth will not be throttled. You are proven to be human if you are not resource hungry.

The currency of stake is measured in how fast the server response, which for me is pretty logical. If bots are swarming your server, the server will be definitely take longer to reply, which makes "the currency" expensive. In this way, during the peak DDOS, your legitimate users can enjoy your site unthrottled yet your server is gonna stay active.

For technical rules, the weight is measured by how long the server send TTFB after all requests is sent. The weight will not be credit only if server sends 5xx or dropping the connection. Websocket connection will simply passthrough.

## Design: How Configuration Calculated

This section helps you undestand how weights work.

Let's set up a hyphotetical endpoints with durations are set here:

```sh
/ -> 0ms
/cheap -> 5ms
/heavy -> 5000ms
```

We create a diagram with this notation
```sh
0s -> /heavy -> 15s (w: 35000)
^-|                            the duration user visit page since first session created
     ^------|                  the url user visit
               ^---|           how much sleepy put the sleep time
                    ^--------| weight value reported from sleepy
```

### Case 1: Default settings

```sh
PENALTY_MULTIPLIER=1
SCRAPER_MULTIPLIER=1
LIMIT_CAP=20000
LIMIT_INITIAL=20000
LIMIT_IDLE_RATE=200
LIMIT_SPEND_RATE=3000
```

```sh
# first, it visit the home page, it is set as LIMIT_INITIAL
0s -> /      -> 0ms   (w: 20000)
# then, it goes heavy: 5s from server + 15s sleeping
# 20000 (initial w) + 5s * 3000 (LIMIT_SPEND_RATE) = 35000
0s -> /heavy -> 15s   (w: 35000)
# at this time, w has reduced by 15s * LIMIT_IDLE_RATE, 
# but this still > LIMIT_CAP, so it still delayed
# 35000 (initial w) - 15s * 200 (LIMIT_IDLE_RATE) = 32000
15s -> /     -> 12s   (w: 32000)
# let's wait another minute (yes, this still get delayed)
# 32000 (initial w) - 45s * 200 (LIMIT_IDLE_RATE) = 23000
60s -> /     -> 3s    (w: 23000)
# by the next minute there's no delay, but this is lower than < LIMIT_CAP
# this user now said to be "trusted", until it get spammy again
# 23000 (initial w) - 60s * 200 (LIMIT_IDLE_RATE) = 11000
120s -> /    -> 0s    (w: 11000)
```

### Case 2: DDOS-ed settings

We now set PENALTY_MULTIPLIER higher

```sh
PENALTY_MULTIPLIER=3
SCRAPER_MULTIPLIER=1
LIMIT_CAP=20000
LIMIT_INITIAL=20000
LIMIT_IDLE_RATE=200
LIMIT_SPEND_RATE=3000
```

```sh
# this user just entered /heavy without any cookies, what it mean
# they does right away penaltied: LIMIT_INITIAL + 5s from server + 15s * 3 sleeping
# 20000 (initial w) + 5s * 3000 (LIMIT_SPEND_RATE) = 35000
0s -> /heavy -> 50s   (w: 35000)
# at this time, w has reduced by 50s * LIMIT_IDLE_RATE, 
# but this still > LIMIT_CAP, so it still delayed
# 35000 (initial w) - 50s * 200 (LIMIT_IDLE_RATE) = 25000
50s -> /     -> 15s   (w: 25000)
# as we can see this bot get penalized and they just gonna better to drop the cookie
```

### Case 3: Well what if I just drop my cookie

We will now see how SCRAPER_MULTIPLIER works


