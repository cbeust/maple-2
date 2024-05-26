
# War stories

## The Ultima IV bug

Since there was some suspicion toward memory switching, I started by playing with my various memory banks and filling
them with values to see if there was any effect. I picked up a clue when I filled the Bank 2 of the high ram with $FF
and got the following result

The game reads values from that bank, values that it probably expects to have stored there itself earlier, but instead,
it finds my $FF, so clearly, it wasn't able to write there successfully. The game is doing some juggling with the memory
banks in that location:

I pored over my bank switching code, hoping to find a banks[0] instead of a banks[1] or something to that effect, but
came out empty. And again, a2audit passes so the bug is likely to be somewhere else. So I browsed another area of my
bank switching and came across the following logic:
```
// PRE-WRITE is set by odd read access in the C08X range.
// It is reset by even read access or any write access in the $C08X range.
if (address & 0xc080) == 0xc080 {
let odd = (address & 1) == 1;
```

I stared and stared and stared, and suddenly, it hit me...
I replaced the test with

```
if (0xc080..=0xc08f).contains(&address) {
```

and...

TADA!

In case it didn't jump at you, (address & 0xc080) == 0xc080 is true for a lot more values than just the range
$C080..$C08F... Because of this bug, I was resetting PREWRITE A LOT, which caused most writes to the LC to fail.
And this also explains why a2audit didn't catch this bug, which can only be reproduced by accessing some random
addresses above $C080, and even though@Zellyn 's code probably does this now and then, it resets the switches
all the time in-between tests, which causes the bug to remain hidden..
