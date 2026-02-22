# Jerusalem ⚔️

This is under development, if you wanna know about the progress, keep reading. 

## What it supports right now

| Command | Category | Status | Notes |
| :--- | :--- | :--- | :--- |
| SET | String | Supported | Now supports EX (seconds) for TTL |
| GET | String | Supported | Functional, but does not display TTL remaining yet |
| DEL | String | Supported | Specifically for string keys |
| APPEND | String | Supported | Working |
| INCR | String | Supported | Atomic increment for numeric strings |
| PING | Connection | Supported | Does not support pipelining |

## Issues

No issues other than being incomplete

## My order of operations for this project

Currently working on operations for data types like hashmap, lists, and eventually sets.

As for the PING command pipelining, for some reason when you pipeline it, the PING command array is sent without the *. And I honestly have no clue why they have done it like this, surely there must be good reason. The reason why I don't support it, because (main reason is that I don't it makes sense because in the real wrold no one's gonna do that (I don't think)) the code would get a bit messy.

## Maybe plans

Sharding

