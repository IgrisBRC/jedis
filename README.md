# IgrisDB ⚔️
Alright, this time, it's actually me, a real human writing this and not AI. Honestly I don't know why I even chose to have it written by AI to begin with because it looked bloody abysmal. Anyway, uhh yeah this is still under development, if you wanna know about the progress, keep reading. 

## What it supports right now

Only some string operations are supported right now.
Supports SET but not with the EX yet. 
Supported GET, but no TTL yet.
Supports DEL for strings only obviously.
Supports PING, but it doesn't work if you are gonna pipeline the PING command (keep reading for the reason).

## My order of operations for this project

Next steps are to support all the commands for string atleast, which are INCR, DECR, EXISTS, and append.
And after that to move onto the operations for other data types like hashmap, lists, and eventually sets.

As for the PING command pipelining, for some reason when you pipeline it, the PING command array is sent without the *. And I honestly have no clue why they have done it like this, surely there must be good reason. The reason why I don't support it, because (main reason is that I don't it makes sense because in the real wrold no one's gonna do that (I don't think)) the code would get a bit messy.

## Maybe plans

Support for sharding

