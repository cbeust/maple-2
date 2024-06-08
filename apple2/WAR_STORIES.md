
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

## Wizardry 1

My initial implementation didn't boot this game (ended up with a garbage text screen). Looking into the
code revealed a protection that counts nibbles on tracks 10, 11, 12, and 13 ($8A72). However, I couldn't
find anywhere where these counts were checked. @qkumba explained to me that the protection check is actually
done in Pascal (the language Wizardry 1 is written in) so the check would require a P-Code disassembler
to look into. The check allows some variations in these numbers, and at long as they are within these
ranges, the protection passes.

But the weird part about this protection is that even though it does the exact same thing for these four
tracks, the nibble counts expected were very different for all four tracks, e.g. : $120c, $58e, $120e, $53e.
It took me a while to realize that on top of the nibble count, this was also a cross track synchronization
check, and I realized that I had a bug in my code in that area.

In order to simulate the head of the drive correctly, it's important to make sure than when the head
moves to a different track, it needs to remain at approximately the same location, and my code was resetting
it to 0 at each change. Instead, the position of the head needs to be recalculated according to the
length in bits of the current track and the length in bits of the new track. This code is actually explicitly
called out in the [Woz 2.1 reference](https://applesaucefdc.com/woz/reference2/) and looks like this:

```rust
new_position = current_position * new_track_length / current_track_length
```

The last obstacle I faced was that I was not claiming that the disk is write protected. Wizardry 1's disk
will refuse to boot if it detects that it's writable, so make sure to honor the write protect status
for that disk (i.e. $C08D followed by $C08E needs to return a value >= 0x80).

Once this was fixed, the disk booted correctly.
