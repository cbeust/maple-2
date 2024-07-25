## SmartPort

Not really. You need three things in the CX00 page:
Enough bytes to pass a card ID check.
https://github.com/badvision/lawless-legends/blob/41d3ccce650a242a2be2182974d38419[…]ce/src/main/java/jace/hardware/massStorage/CardMassStorage.java

2. The disk capacity (in blocks)
3. The offsets of where your smart port routines can be accessed in your firmware.
   2 and 3 are implemented here:
   https://github.com/badvision/lawless-legends/blob/41d3ccce650a242a2be2182974d38419[…]ce/src/main/java/jace/hardware/massStorage/CardMassStorage.java
   I get around the other needs by simply writing my own “Boot 0” loader if C700 is executed directly. You basically
   just load the first block at 800 and JMP to
   it. https://github.com/badvision/lawless-legends/blob/41d3ccce650a242a2be2182974d38419[…]ols/jace/src/main/java/jace/hardware/massStorage/LargeDisk.java
   If you want to implement your own virtual block emulation, it gets tricky because you have to deal with prodos file
   structures. I do a read-only thing that can mimic a block drive but with a folder of regular files — but I don’t
   advertise the feature because it’s kind of hard to get working properly. You need all the files to have ciderpress
   style names with the A$ offsets in the file names. (edited)

## Sound

Brendan:

ok so there ’s a very easy “cheap” way to do speaker sound. You just count the number of times the speaker toggles in a
given interval (it’s about 22-23 cycles per 44.1khz sample, but anyway you can just divide and round, etc) — the number
of times the speaker is toggled is multiplied by a volume (say, 400) and then that is the next audio sample to output.
When you do this, the sound output itself is a blocking thing so as you output sound it will force emulator to time at
the 1mhz because it will block for audio buffers to free up at some point. But basically, that’s all you need for
speaker sound.

https://github.com/badvision/jace/blob/main/src/main/java/jace/apple2e/Speaker.java



## List of WOZ disks that require FLUX support

https://archive.org/details/wozaday_Bandits
https://archive.org/details/wozaday_Cyclod
https://archive.org/details/wozaday_Fly_Wars
https://archive.org/details/wozaday_Jellyfish
https://archive.org/details/wozaday_Lemmings
https://archive.org/details/wozaday_Minotaur
https://archive.org/details/wozaday_Return_of_Heracles

## Crates for concurrent access

- [left-right](https://lib.rs/crates/left-right)
Left-right is a concurrency primitive for high concurrency reads over a single-writer data structure. The primitive
keeps two copies of the backing data structure, one that is accessed by readers, and one that is access by the (single)
writer. This enables all reads to proceed in parallel with minimal coordination, and shifts the coordination overhead to
the writer. In the absence of writes, reads scale linearly with the number of cores.


- [https://docs.rs/watch/latest/watch/](Watch)
This crate provides a synchronous message passing channel that only retains the most recent value.

## Disk Controller

From the WOZ spec:

As referenced in the previous section, when you access $C088,X it begins the timer to turn off the drive motor. But, it
also returns the value of the data latch! Why would it do that?!? It does this for one simple reason, the low bit of the
address line (A0) is connected through a NOT gate to the Output Enable line of the data latch. Therefore every soft
switch on an even address should actually return the value of the data latch.

Another nuance that needs to be implemented is that reading $C08D,X will reset the sequencer and clear the contents of
the data latch. This is used to great effect in the E7 protection scheme to resynchronize the nibble stream to make
timing bits become valid data bits.

## Memory init (necessary for a few protections, e.g. BugAttack)

for i in 0x000..0xC000 {
r.mb_ram[i] = ((((i+2) >> 1) & 1)*0xFF) as u8 ;
}
for i in 0x400..0xC00 {
r.mb_ram[i] = (i & 0xF) as u8;
}

et y'a aussi :
self.mb_ram[0x3F4] = 0xFF;
self.mb_ram[0x3F3] = 0xFE;

    for i in 0xD000..0xFFFF {
        self.mb_ram[i] = 0;
    }


## AppleWin trace

From the AppleWin debugger: tf "filename" [v]
Trace to file [optionally with video scanner info]
(Sorry not documented in the .chm help file)

## Graphics

Kent Dickey:

I suggest you begin with a simple coloring algorithm for hires: from even pixels, look at the preceding pixel, and the
next pixel, so you look at 3 pixels. One HGR pixel at 280 pixel resolution becomes two 560-pixel pixels, we'll call
this "HGR" pixels and "fine" pixels. Assume we are at an even pixel position, hi bit is clear. So HGR bits 000 colors
the middle pixels as two black fine pixels;

001 (meaning prev=0, this=0, next=1) becomes 2 black fine pixels;
010 is two green fine pixels,
011 is 2 white fine pixels
100 is 2 black fine pixels
101 is 2 green fine pixels,
110 is 2 white fine pixels
111 is 2 white fine pixels.

For odd pixel positions, replace green with purple. For hi bit set, replace
green with orange for even columns; with blue for odd.


This is a basic HGR coloring scheme. Then, get more complex with
the "half-bit shift" (which is one "fine" pixel) when the hi bit is set. Once you get this, you're ready for browns and
yellows. I may have reversed colors in this description, but it will be obvious when you implement it.

As for what RGB values green,violet,orange and blue are: easy first pass is run another emulator, run a pixel
magnifier (on a Mac, I run ColorSync Utility, and click the magnifying glass), and write down what they are using. Even
better, turn an Apple II on, get the colors on an Apple II monitor that you like (make sure it's tuned to not be too
weird), take a photo, and then look at the RGB values of the photo (make sure to scale so black is 0 and white is the
maximum, the photograph will pick arbitrary values for black and white depending on environmental factors). If you go
the photo route, you will have more accurate color than most emulators.

For the photo route, you have to take a photo of your modern screen, too, and make sure the RGB values match (this
solves the gamma problem). Adjust your code so that the photo of the screen matches the photo of the Apple II.

---

Brendan:

The explanation in wikipedia and Nibble magazine articles are “correct” but in the sense that they were trying to take a
model with complex behavior and explain it in terms of it’s stable state. The model that says HGR and DHGR are 140
pixels wide are correct in the sense that they are trying to explain the most stable state that has a reproducible
outcome (unless of course you stick orange next to blue and get a white pixel, but nevermind that for now.) The funky
nature of 560-pixels displayed in color was understood probably better in the 80's by enthusiasts because back then
there were more folks that had a working knowledge of how analog NTSC televisions worked, at least on some level, or had
dabbled with HAM radio experiments around these kinds of things. Nowadays it’s a lot harder to explain because we live
in an RGB world with framebuffers and color-calibrated displays. Therefore it takes a lot more exercise to explain how
to mimic the old analog stuff in our modern context as a result. (edited)

There was no way to reproduce the same color exactly the same on every composite display, and pixels were generally
always kind of fuzzy, so explaining the color fringe wasn’t something we dabbled with a lot back then. (I could see it
as a kid, and I used a magnifying glass to look at it, but I couldn’t explain it to myself…)  And nowadays we have more
powerful tooling to run much more complex models than we could have back then. Converting modern digital pictures to
higher-fidelity DHGR renderings is somewhat anachronistic in that sense, but I applaud it all the same. :wink: (edited)

---

Joshua Bell:

Think of the screen as 560 "screen bits" across. Per the previous discussion, sets of 4 "screen bits" determine the
color. The "screen bit" pattern for green is 0011,0011,... and the "screen bit" pattern for violet is 1100,1100,....

---

S Champailler:

If you like a bit of math, the basic NTSC (QAM) decoding is not that hard, really. Personnally I went down that road
because I wanted to learn some maths while making AccurApple. I understand that you can solve most of the composite
emulation with a table and a few measurements, but the math skills you could learn (if you don't already have them) is
really helpful eleswhere (at least in my job :-)). The issue with the maths is just that they are a model of reality and
you'll soon discover that this model (if you stick with the regular QAM equations) is far away from reality. And if you
want to improve it, you'll need to become a full time CRT engineer :slightly_smiling_face: If you feel lost, I can
explain bits and pieces to you(or, to be more honnest: the one that I think I have understood :-))

---

In HGR 280x192 mode, a "pixel" (x=0...279) is one true memory bit wide, which is doubled to two "screen bits" wide by
the circuitry - hence 280 becomes 560. To make green you want to lay down 0101010,1010101,... as real bits in memory (
only 7 bits per byte are used) which is doubled to 00110011... when generating the "screen bits" and you get a nice
green line.
What the Wikipedia article is trying to say is: if you start with an all black screen (all 0s), and try to set the top
left "pixel" to green, what Applesoft does (for example) is look at what part of the memory bit pattern should be
applied. Since the memory bit pattern for green is 0101... then the leftmost bit is 0 and so set the top left pixel bit
in memory to 0. But ... it's already 0. So nothing happens! Remember, for green, you need the 0101... memory pattern. So
if the screen is black to get that you need to change the second pixel in the row to a 1. Then you'll get 01, and in "
screen bits" that's 0011 and yay you get green!
For violet, the "screen bit" pattern is 1100,1100,... and so the HGR memory bit pattern is 1010.... So when starting
with an all black screen if you set the top-leftmost bit to 1 you will get some violet.
If you were starting with an all white screen (all 1s), to get the green and violet patterns you need to introduce 0s
instead, so which bits you need to touch are the opposite.
So the Wikipedia article is a very very coarse approximation that only applies if you're thinking of the screen in a way
typical for Applesoft BASIC programmers -- drawing pixels with HPLOT on a black hires screen -- but not a valid
description of how the underlying screen generation works
Here's the mental model I use: the NTSC decoder - based on 1950s tube technology - is trying to determine the color (how
much intensity to apply to the R/G/B guns) by watching the intensity of the signal across the scan line in sync with the
color clock. A signal that looks like 1000,1000,1000,1000,1000 is a thin signal mostly aligned with the color clock's
blue phase, which results in the blue gun firing, but not at full intensity, so you get dark blue. A signal that looks
like 0100,0100,0100,0100 results in the green gun firing (ditto). A signal that looks like 0001,0001,0001,0001 results
in the red gun firing (ditto). 0010,0010,0010,0010 is dark green+ dark red = dark yellow (colors are weird!), which we
call brown. This is only vaguely approximately correct, but bear with me. Generate 1100,1100,1100,1100 and you'd expect
blue and green to fire and indeed you get brigher blue and some green mixed in. Generate 1110,1110,1110,1110 and you'd
expect full blue, full green, and a little yellow and indeed you're getting cyan.

The TV interprets the bits based on a 4-bit window. 0001,0001 is very different from 0010,0010, where I've placed the
commas after an "aligned" 4-bit unit. In the pattern 0001,0001, repeating, in general, the screen will appear to be a
single "solid" color. And 0010,0010 is a different "solid" color. Even though the pattern is three 0's then one 1,
repeating, the alignment of the pattern to an aligned 4-bit "unit" matters. To the TV, an aligned group of 4 bits are
a "unit" of a sort, and the pattern of bits in that unit determine the color. The number of bits set determine the
intensity (how brightly lit it is). And: the previous and next "unit" affect the color/intensity (but probably not
further away). There are NTSC formulas, and they can be useful, but they are assuming smooth-ish sine waves, and the
Apple II output is mostly square waves, so they aren't quite right. And here's where things get complex and require
undergrad level EE signal processing. So, at this point, give up on theory and follow some other program which you like
the look of.

This. Can’t agree more. At some point I was dreaming of doing this in shaders. When I learned more about it I ditched
the idea. Shaders are parallel, this is inherently serial.

Think of the screen as 560 "screen bits" across. Per the previous discussion, sets of 4 "screen bits" determine the
color. The "screen bit" pattern for green is 0011,0011,... and the "screen bit" pattern for violet is 1100,1100,....
In HGR 280x192 mode, a "pixel" (x=0...279) is one true memory bit wide, which is doubled to two "screen bits" wide by
the circuitry - hence 280 becomes 560. To make green you want to lay down 0101010,1010101,... as real bits in memory (
only 7 bits per byte are used) which is doubled to 00110011... when generating the "screen bits" and you get a nice
green line.

What the Wikipedia article is trying to say is: if you start with an all black screen (all 0s), and try to set the top
left "pixel" to green, what Applesoft does (for example) is look at what part of the memory bit pattern should be
applied. Since the memory bit pattern for green is 0101... then the leftmost bit is 0 and so set the top left pixel bit
in memory to 0. But ... it's already 0. So nothing happens! Remember, for green, you need the 0101... memory pattern. So
if the screen is black to get that you need to change the second pixel in the row to a 1. Then you'll get 01, and in "
screen bits" that's 0011 and yay you get green!

For violet, the "screen bit" pattern is 1100,1100,... and so the HGR memory bit pattern is 1010.... So when starting
with an all black screen if you set the top-leftmost bit to 1 you will get some violet.

If you were starting with an all white screen (all 1s), to get the green and violet patterns you need to introduce 0s
instead, so which bits you need to touch are the opposite.

So the Wikipedia article is a very very coarse approximation that only applies if you're thinking of the screen in a way
typical for Applesoft BASIC programmers -- drawing pixels with HPLOT on a black hires screen -- but not a valid
description of how the underlying screen generation works.

# Double hires (DHGR)

AppleWin discussion:  https://github.com/AppleWin/AppleWin/issues/764#issuecomment-590133459

```6502
LDA $C050  TEXToff
LDA $C057  HRon
LDA $C052  MIXEDoff
STA $C00C or $C00D
STA $C05E
STA $C05F
STA $C00C or $C00D
STA $C05E
STA $C05F
STA $C05E   DHGRon
STA $C00D 80col
```

```
C00C+C00C (00) = DHGR BW (AppleColor, EVE, Feline) - AppleWin ok
C00D+C00D (11) = DHGR color (AppleColor, EVE, Feline) - AppleWin ok
C00C+C00D (01) = DHGR mixed (AppleColor, Feline) - AppleWin ok
C00D+C00C (10) = 160x192 (AppleColor)
```

More DHGR discussions: https://github.com/AppleWin/AppleWin/issues/254#issuecomment-67205861


- Black Cauldron
- King's Quest
- Dragon Wars
- Star Trek First Contact


Write to $C005 to enable aux write

DHR loader:
https://github.com/a2-4am/a2fc.system


```
sta IO.SETHIRES #C057
sta IO.SETMIXED #C052
sta IO.CLRTEXt  #C050

lda IO.RDIOUDIS #C07E

sta IO.CLR80DISP #C00C
sta IO.SETAN3    #C05E
sta IO.CLRAN3    #C05F
sta IO.SETAN3
sta IO.CLRAN3

sta IO.SET80DISP #C00D
sta IO.SETAN3
sta IO.CLRAN3
sta IO.SETAN3

bmi .1
sta IO.CLRIOUDIS #C07F
.1
sta IO.SET80STORE #C001
```

## Memory banks

80Store does not allow access to all of Aux memory. It only affects the way $C054 and $C055 work.
With 80Store off (via STA $C000), then $C054 allows viewing of text page#1 and hi-res graphics screen #1.  
And $C055 allows viewing of text page #2 and hi-res screen #2.
With 80Store on, (via STA $C001), then $C054 allows
certain instructions access (reading/writing)(not viewing) to Main memory of page #1 of the text screen and page #1
of the hi-res graphics screen, and $C055 allows some instructions for reading/writing of the Aux memory pages of text
page 1x and hi-res graphics screen 1x.
Memory outside the range of the text screen ($400.7FF) and hi-res graphics screen ($2000.3FFF) are not affected by
those instructions. But a combination of flipping the softswitches in a certain order will also allow instructions to
read/write to the 2nd text screen and 2nd hi-res screen of Aux memory. I wrote a demo displaying Dbl-hi-res page #2
while drawing on dbl-hi-res page #1 and page flipping between them.

## Night Mission (Woz v1 disk)

- Transfers control from $804 to $304.
- Next track
- Waits for $DB $AB $BF
- Reads 2 bytes, stores in $B2FE $B2FF
- Then 13 * 256 bytes in $B300..$BFFF
- Expects to find #$DB (test at $36C)
- Diagram of track 1 (phase 3) and above:

```
DB AB BF FB EB...        DB
         |_____________| |____ Has to end with this
             2 + 13 * $FF bytes, filling $B2FE..$BFFF
```

If $DB is not found, the upper left corner of the text screen ($400) is filled with incrementing characters until the
code just gives up.

My code was sometimes returning an illegal nibble ($F0) in the nibble stream. The bug was in my Woz1 implementation
which was not wrapping at the end of the stream correctly. 

## Frogger

The wozaday Frogger image on boot starts by accessing every 1/4 track from 0.00 to 8.75 and then goes back down again
by 1/4 tracks.

It's checksumming slot ROMs which it checks later that they haven't changed (i.e., memory capture image used on another
machine), $F0 and $F8 ROM. If you're seeing memory being zeroed then it has passed that check
if you reach $B72B then it's reading the disk
What's in $00 and $01?  They are current and requested phases for the disk.
$01 /4 is the whole track number, %4 is the quarter-track number.
qkumba
okay, if you get to $B75A then it's found the address marker and is about to read the actual sector.
If you get to $B7A9 then it read the sector and now it's checking the epilogue
$A008 holds the error code on reboot.

The next check I'm failing is at $B7F4 (which stores #$14 in $A008 and returns, proceeding to the reboot area). This is
part of a complicated loop that seems to read and decode nibbles but which wants successive EOR's with the previously
calculated values to be equal to 0. I pass this 16 times and then fail. I am guessing I'm on the wrong quarter track,
that will be my next investigation
$00 and $01 are both equal to 0, which means it's trying to read this from T0, but I notice that my phase
is in the 5-6 range, so it's lagging. This gave me an idea: whenever the head is being asked to move or to spin
down, I delay if it's not in motion already. What if there was a bug there? So I completely removed the
delay and...

The next bug was that the keyboard didn't seem to work. Disassembling what the game is doing, it's actually
not checking $C000 but $C0xx with xx changing values.

9C3E: A9 FF      LDA #$FF      |             
9C40: 85 3A      STA $3A       |       A$003A
9C42: A9 BF      LDA #$BF      |             
9C44: 85 3B      STA $3B       |       A$003B
9C46: A0 13      LDY #$13      |             
9C48: D1 3A      CMP ($3A),Y   |  a$C012:v$00
9C4A: A0 07      LDY #$07      |             
9C4C: B1 3A      LDA ($3A),Y   |  a$C006:v$00
9C4E: C9 A0      CMP #$A0      |             
9C50: D0 02      BNE $9C54     |             
9C54: C9 C4      CMP #$C4      |

After I updated my code to reflect keys pressed into all these memory locations, everything is working fine and I can
redefine keys, play the game, etc...

Interestingly, Maze Craze Construction Set is now booting, it's probably using a weaker version of Spiradisc. Lunar
Leeper and Frogger are still not booting.
