# sleepy (WIP)

This simple proxy creates a HTTP-based rate limit via an injected `proxy-session` token, then applies rate limit based on configurable dynamic leaky bucket algorithm.

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

# accept float, by default the server response time will be multiplied by this
PENALTY_MULTIPLIER=3
# accept bool, whether the limit weight is emitted as `X-Sleepy-Weight`
EMIT_HEADERS=true

# all rate limit units below in miliseconds
LIMIT_CAP=20000 # how much computation time is allowed before penalty kicks in
LIMIT_INITIAL=20000 # initial weight for new visitors, usually you want this the same as limit cap
LIMIT_IDLE_RATE=1 # how much weight per milisecond the limit rate is go down by no traffic
LIMIT_SPEND_RATE=3 # how much weight per milisecond the limit rate is go up by server spending
```

## Design: Weight Scoring

Actually, this software is a (\*my) response of recently viral [AI Bot Firewall](https://github.com/TecharoHQ/anubis), but some people (me too) might consider it a bit uneasy that to fight a bot we have to make our devices slightly a bit warm. And then i'm inspired with the way ethereum solve the energy dilemma against cryptocurrency system by embrancing Proof of Stake rather than Proof of Work.

In this concept, the proof of stake is in the cookies: As long as the cookie bearer is proven humane, the server bandwidth will not be throttled. You are proven to be human if you are not resource hungry.

The currency of stake is measured in how fast the server response, which for me is pretty logical. If bots are swarming your server, the server will be definitely take longer to reply, which makes "the currency" expensive. In this way, during the peak DDOS, your legitimate users can enjoy your site unthrottled yet your server is gonna stay active.
