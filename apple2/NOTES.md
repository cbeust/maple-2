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


## A2Audit

To know which test is being run:

```rust
if pc == 0x61d9 {
   println!("Starting a2audit test {:02X}{:02X}", self.memory.get(0x1fd), self.memory.get(0x1fc) - 2);
}
```

Data-driven tests start at $61AE.

## Sound

Brendan:

ok so there ’s a very easy “cheap” way to do speaker sound. You just count the number of times the speaker toggles in a
given interval (it’s about 22-23 cycles per 44.1khz sample, but anyway you can just divide and round, etc) — the number
of times the speaker is toggled is multiplied by a volume (say, 400) and then that is the next audio sample to output.
When you do this, the sound output itself is a blocking thing so as you output sound it will force emulator to time at
the 1mhz because it will block for audio buffers to free up at some point. But basically, that’s all you need for
speaker sound.

https://github.com/badvision/jace/blob/main/src/main/java/jace/apple2e/Speaker.java

Wiz:

En première approximation,  tu enregistres le fait qu'un tick de c030 met le son à un et le tick suivant
le met à zéro => donc tu as une onde avec des paquets de 1 suivi de paquets de zéros. Tu fais ensuite ce qu'on
appelle une "décimation" : tu relis ton signal de  1MHz "tous les" 44Khz (bref, tu lis 1 sample de ton signal 1MHz
tous les 1_000_000/44_1000).

Vu comment les musiques apple sont conçues, ça marchera déjà pas mal.
Ensuite, le gars de KEGS a une routine plus sophistiquée qui fait tout ça "on the fly" et en plus de
manière numériquement propre. Le gars a décrit son algo et j'ai reproduit
ça ici: https://gitlab.com/wiz21/accurapple/-/blob/main/Documentations/maths.pdf section 6.1

Si tu veux aller plus loin, il faut absolument comprendre Fourrier et les filtres passe bas.
Ensuite, y'a le niveau encore au dessus où on simule le fait que la réponse du speaker à l'impulse dépend
des impulse précédentes. Là, cf. le même PDF où je donne toutes les maths du "simple harmonic oscillator".
Mais là, ça ne fait une différence que dans très peu de cas => gros coupage de cheveux en quatre pour pas
grand chose (mais c'est très fun à comprendre !!!!)
Enfin, le speaker a une réponse fréquentielle bien à lui et pour la reproduire il faut passer encore par
une sorte d'équalisation. Pour le momen j'utilise Fourrier pour ça mais je pense que c'est pas correct point
de vue théorique. Ensuit,e pour le faire, il faut enregistrer la réponse fréquentielle en question. J'en suis
là car pour ça il faut un studio d'enregistrement et des gens compétents pour faire ça (idéalement, il faut
une chambre anéchoïque et des micros qui coûtent des bagnoles). Mais même avec ça, on est encore loin du
compte car en fait ce que les gens ont dans l'oreille, ce n'est pas le son de leur Apple mais le son de leur
Apple dans leur bureau. Et la pièce change énormément la perception qu'on a du son 🙂
Et tout ça, c'est des trucs que j'ai appris sur le tas... Un ingé son avec le bagage mathématique
y trouverait sans doute bcp à redire !

Les jeux à tester: Prince of Persia,  SeaDragon, Archon, Goonies, Bruce Lee...

## Mockingboard

https://gswv.apple2.org.za/a2zine/Docs/Mockingboard_MiniManual.html

AY datasheet
https://apple2infinitum.slack.com/files/U05RUNFFQMD/F07GKL84W1H/ay-3-8910-8912_feb-1979.pdf?origin_team=T1J8S1LGH&origin_channel=CABEM8JFK

Ian Brumby
When reading the schematic, one of the key things is to see where A0, A1, A2, A3 and A7 are mapped to (on the 6522's).
A0, A1, A2, A3, A7 are the Apple II address lines. The Mockingboard is accessed (assuming slot 4) in $C400-$C407 or
$C480-$C487. The $0-$7 are A0-A3 and the $80 is A7. Here you can see that it doesn't care about A4, A5, A6 for its
decoding. You can see that it maps the low 4 bits to RS0-RS3 on the 6522's. And A7 controls which 6522 is used.
The schematic (GarberStreet Mockingboard Clone) maps the left 6522's IRQ to the Apple II's IRQ. It maps the right 6522's
IRQ to the Apple II's NMI. Note that this is not "correct" (not what most Mockingboards do). The updated Mockingboard
clone that ReactiveMicro sells has fixed this (maps both to Apple II IRQ).
Use mb-audit (https://github.com/tomcw/mb-audit) to test your code when you are far enough along.
Note that a lot of Mockingboard software use a similar auto-detection routine, which uses the 6522's timers to determine
if there is a Mockingboard in the system. You may choose to start with implementing the 6522's timers first because if
this check fails then the software will assume there is no Mockingboard in your system.

ct6502
Yes, agree with everything that @Ian Brumby
says. One minor note is that I tried testing with mb-audit but still don’t completely pass all of the tests. I think
there are a lot of subtle tests, especially of the 6522. My emulator seems to work with all Mockingboard software out
there, so I’m not super worried. I would like to go back and fix it so it passes all the tests. I would say don’t go to
heroic lengths when you’re first developing Mockingboard support to try to make all of those tests pass - just circle
back later once you’ve got music + sound effects working properly.

rikkles
If you'd like, take a look at my implementation. I don't have the interrupts (handled by the physical bus card) but I have all the logic of the Mockingboard in 400 lines of C++. This includes the UI component and the de/serialization. The core of it is maybe 80 lines.
The AY code (also C++, adapted from the Peter Sovietov C codebase) is independent.
https://github.com/hasseily/SuperDuperDisplay/blob/main/MockingboardManager.cpp

Samir Sinha
I'll post my implementation as well (tried to reference my sources in the code.)   But it currently only works for a few
titles. And it doesn't pass the most recent mbaudit.
https://github.com/samkusin/clemens_iigs/blob/main/devices/mockingboard.c

Brendan
The hardest part about implementing Mockingboard (other than adding interrupts to your emulator, etc) is the timers, and
making sure the cycle accuracy of the timers is in lock-step with the CPU. This is because mockingboard detection on
several games is based on turning on the timer, waiting a few cycles, and then reading the timer looking for a specific
value to be present, which is an indicator that there is a VIA running at that location and therefore it is a
mockingboard (or compatible)

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

## Floppy drawing

fast-image-resize

Visualization:
https://github.com/dbalsom/fluxfox/blob/main/src/visualization/mod.rs


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

F-15 Strike Eagle
=================

xot:
Hardcore Computist #24 talks about F-15 Strike Eagle hiding the RWST in the RAM card. There are multiple versions
however. Hardcore Computist #35 has a different crack and it also points to issues #28 and #29 regarding other
versions. (edited)

http://computist.textfiles.com/ISSUE.24/page-19.jpg

stivo:
```
loadTrack() woz2   drive:1 tmap track:0 (0.00)
loadTrack() woz2   drive:1 tmap track:3 (0.75)
loadTrack() woz2   drive:1 tmap track:7 (1.75)
loadTrack() woz2   drive:1 tmap track:5 (1.25)
loadTrack() woz2   drive:1 tmap track:1 (0.25)
loadTrack() woz2   drive:1 tmap track:3 (0.75)
loadTrack() woz2   drive:1 tmap track:7 (1.75)
loadTrack() woz2   drive:1 tmap track:8 (2.00)
loadTrack() woz2   drive:1 tmap track:11 (2.75)
loadTrack() woz2   drive:1 tmap track:15 (3.75)
loadTrack() woz2   drive:1 tmap track:19 (4.75)
loadTrack() woz2   drive:1 tmap track:23 (5.75)
loadTrack() woz2   drive:1 tmap track:27 (6.75)
loadTrack() woz2   drive:1 tmap track:31 (7.75)
loadTrack() woz2   drive:1 tmap track:35 (8.75)
loadTrack() woz2   drive:1 tmap track:39 (9.75)
loadTrack() woz2   drive:1 tmap track:43 (10.75)
```

Final fix: requires a perfect implementation of the LC. Even though I was passing `a2audit`, there were still bugs
in my LC implementation that caused F-15 to not boot.

Mr.Do
=====

Need to support the undocumented opcode $74 and also return the latch from $C088,X (instead of the usual $C08C,X).

Maniac Mansion
==============

The game uses the odd $C011 switch to make a few language card decisions, so you need to implement it. Sather
says it represents the opposite of the bank1/bank2 switch. This fix allowed the game to boot:

```rust
0xc011 => {
    result = Some(if self.bank1 { 0 } else { 0x80 })
}
```
