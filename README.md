# IgrisDB ⚔️
Alright, this time, it's actually me, a real human writing this and not AI. Honestly I don't know why I even chose to have it written by AI to begin with because it looked bloody abysmal. Anyway, uhh yeah this is still under development, if you wanna know about the progress, keep reading. 

## What it supports right now

| Command | Category | Status | Notes |
| :--- | :--- | :--- | :--- |
| SET | String | Supported | Now supports EX (seconds) for TTL |
| GET | String | Supported | Functional, but does not display TTL remaining yet |
| DEL | String | Supported | Specifically for string keys |
| APPEND | String | Supported | Working, but susceptible to stalls at high volume |
| INCR | String | Supported | Atomic increment for numeric strings |
| PING | Connection | Supported | Does not support pipelining |

## Issues

The database as of now stalls for SET operation at 1_000_000 requests I don't know exactly why.
By stalling I mean it doesn't deadlock, the database eventually does finish, which rules out a deadlock, however, it is like 50 times slower for some reason. 

For example SET finishes at 100k RPS for 200_000 requets but at 4k RPS at 1_000_000

A similar 'stall' exists for APPEND, but at an even lower number like 70k requests. I don't know exactly why these 2 commands seem to have these isseus.
My speculation is that something to do with the channel queues.

## My order of operations for this project

Because of the stalls mentioned in the above issues I have decided to implement a 2 choir (2 threadpool instances, one for receiving requests, the other for sending responses) setup for this project to avoid channels getting flooded.

And after that to move onto the operations for other data types like hashmap, lists, and eventually sets.

As for the PING command pipelining, for some reason when you pipeline it, the PING command array is sent without the *. And I honestly have no clue why they have done it like this, surely there must be good reason. The reason why I don't support it, because (main reason is that I don't it makes sense because in the real wrold no one's gonna do that (I don't think)) the code would get a bit messy.

## Maybe plans

Support for sharding

