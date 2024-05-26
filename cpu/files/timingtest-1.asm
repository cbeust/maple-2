; Instruction timings for 6502 emulator debugging
; Exercise all instructions and all cases of page crossing
; For discussion, please see http://forum.6502.org/viewtopic.php?f=8&t=3340
;
; There's no timing done in this code: use a hardware timing
; or instrumentation in your emulator of choice.
;
; BRK is not included for portability reasons.
;
; Assembly syntax is for Michal Kowalski's simulator as found at
;    www.exifpro.com/utils.html
;
; Using version 1.2.11 of that program, this version executes in
;    1130 clocks
; But visual6502, which should be a more trusted reference, executes in
;    1141 clocks
; See http://goo.gl/956Cxi
;
; When modifying, take care that all far branches still cross a page boundary.
; See labels 'far1' and so on.
;
; Using http://www.llx.com/~nparker/a2/opcodes.html as instruction reference
;
; Copyright (C) 2015  Ed Spittles
;
;    This program is free software; you can redistribute it and/or modify
;    it under the terms of the GNU General Public License as published by
;    the Free Software Foundation; either version 2 of the License, or
;    (at your option) any later version.
;
;    This program is distributed in the hope that it will be useful,
;    but WITHOUT ANY WARRANTY; without even the implied warranty of
;    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;    GNU General Public License for more details.
;
;    You should have received a copy of the GNU General Public License along
;    with this program; if not, write to the Free Software Foundation, Inc.,
;    51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

; any nonzero byte should work here so long as we avoid poking delicate hardware
zdummy=$17
dummy=$1717

 .ORG $1000
start:

init:
 LDX #$FF
 TXS
 LDX #1
 STX zdummy    ; zdummy points to $0101
 STX zdummy+1
 STX zdummy-1  ; zdummy-1 also points to $0101 - that's zdummy,X with X=$FF

transfers:
 TXA
 TAY
 TYA
 TSX
 TAX
 
arithmetic:
 LDX #0 ; non-page-crossing
 LDY #0
 ORA (zdummy,X)
 ORA zdummy
 ORA #0
 ORA dummy
 ORA (zdummy),Y
 ORA zdummy,X
 ORA dummy,X
 ORA dummy,Y
 DEX ; page crossing or wrapping
 DEY
 ORA (zdummy,X)
 ORA (zdummy),Y
 ORA zdummy,X
 ORA dummy,X
 ORA dummy,Y

 INX ; non-page-crossing
 INY
 AND (zdummy,X)
 AND zdummy
 AND #0
 AND dummy
 AND (zdummy),Y
 AND zdummy,X
 AND dummy,X
 AND dummy,Y
 DEX ; page crossing or wrapping
 DEY
 AND (zdummy,X)
 AND (zdummy),Y
 AND zdummy,X
 AND dummy,X
 AND dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 EOR (zdummy,X)
 EOR zdummy
 EOR #0
 EOR dummy
 EOR (zdummy),Y
 EOR zdummy,X
 EOR dummy,X
 EOR dummy,Y
 DEX ; page crossing or wrapping
 DEY
 EOR (zdummy,X)
 EOR (zdummy),Y
 EOR zdummy,X
 EOR dummy,X
 EOR dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 ADC (zdummy,X)
 ADC zdummy
 ADC #0
 ADC dummy
 ADC (zdummy),Y
 ADC zdummy,X
 ADC dummy,X
 ADC dummy,Y
 DEX ; page crossing or wrapping
 DEY
 ADC (zdummy,X)
 ADC (zdummy),Y
 ADC zdummy,X
 ADC dummy,X
 ADC dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 LDA #1 ; don't break our pointer in zdummy
 STA (zdummy,X)
 STA zdummy
 STA dummy
 STA (zdummy),Y
 STA zdummy,X
 STA dummy,X
 STA dummy,Y
 DEX ; page crossing or wrapping
 DEY
 STA (zdummy,X)
 STA (zdummy),Y
 STA zdummy,X
 STA dummy,X
 STA dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 LDA (zdummy,X)
 LDA zdummy
 LDA #0
 LDA dummy
 LDA (zdummy),Y
 LDA zdummy,X
 LDA dummy,X
 LDA dummy,Y
 DEX ; page crossing or wrapping
 DEY
 LDA (zdummy,X)
 LDA (zdummy),Y
 LDA zdummy,X
 LDA dummy,X
 LDA dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 CMP (zdummy,X)
 CMP zdummy
 CMP #0
 CMP dummy
 CMP (zdummy),Y
 CMP zdummy,X
 CMP dummy,X
 CMP dummy,Y
 DEX ; page crossing or wrapping
 DEY
 CMP (zdummy,X)
 CMP (zdummy),Y
 CMP zdummy,X
 CMP dummy,X
 CMP dummy,Y

 LDX #0 ; non-page-crossing
 LDY #0
 SBC (zdummy,X)
 SBC zdummy
 SBC #0
 SBC dummy
 SBC (zdummy),Y
 SBC zdummy,X
 SBC dummy,X
 SBC dummy,Y
 DEX ; page crossing or wrapping
 DEY
 SBC (zdummy,X)
 SBC (zdummy),Y
 SBC zdummy,X
 SBC dummy,X
 SBC dummy,Y

rmw: ; may need to restore our zp pointer value
     ; therefore perform op then inverse op
 LDX #0 ; non-page-crossing
 ASL zdummy
 LSR zdummy
 ASL dummy
 LSR dummy
 ASL zdummy,X
 LSR zdummy,X
 ASL dummy,X
 LSR dummy,X
 DEX ; page crossing or wrapping
 ASL zdummy,X
 LSR zdummy,X
 ASL dummy,X
 LSR dummy,X

 LDX #0 ; non-page-crossing
 ROL zdummy
 ROR zdummy
 ROL dummy
 ROR dummy
 ROL zdummy,X
 ROR zdummy,X
 ROL dummy,X
 ROR dummy,X
 DEX ; page crossing or wrapping
 ROL zdummy,X
 ROR zdummy,X
 ROL dummy,X
 ROR dummy,X

 ; use Y indexing for LDX and STX
 LDY #0 ; non-page-crossing
 LDX zdummy
 STX zdummy
 LDX dummy
 STX dummy
 LDX zdummy,Y
 STX zdummy,Y
 LDX dummy,Y
 DEY ; page crossing or wrapping
 LDX zdummy,Y
 STX zdummy,Y
 LDX dummy,Y

 LDX #0 ; non-page-crossing
 DEC zdummy
 INC zdummy
 DEC dummy
 INC dummy
 DEC zdummy,X
 INC zdummy,X
 DEC dummy,X
 INC dummy,X
 DEX ; page crossing or wrapping
 DEC zdummy,X
 INC zdummy,X
 DEC dummy,X
 INC dummy,X

branches: ; forward, backward, both with and without a page crossing, also not takens
 LDX #0
 BPL br1a ; forward, no page crossing
br1b:
 BPL far1 ; forward, page crossing
br1a:
 BMI br1a ; not taken
 BPL br1b ; backward
return1:

 BEQ br2a ; forward, no page crossing
br2b:
 BEQ far2 ; forward, page crossing
br2a:
 BNE br2a ; not taken
 BEQ br2b ; backward
return2:

 LDX #$FF
 BMI br3a ; forward, no page crossing
br3b:
 BMI far3 ; forward, page crossing
br3a:
 BPL br3a ; not taken
 BMI br3b ; backward
return3:

 BNE br4a ; forward, no page crossing
br4b:
 BNE far4 ; forward, page crossing
br4a:
 BEQ br4a ; not taken
 BNE br4b ; backward
return4:

 CLC
 BCC br5a ; forward, no page crossing
br5b:
 BCC far5 ; forward, page crossing
br5a:
 BCS br5a ; not taken
 BCC br5b ; backward
return5:

 SEC
 BCS br6a ; forward, no page crossing
br6b:
 BCS far6 ; forward, page crossing
br6a:
 BCC br6a ; not taken
 BCS br6b ; backward
return6:

 CLV
 BVC br7a ; forward, no page crossing
br7b:
 BVC far7 ; forward, page crossing
br7a:
 BVS br7a ; not taken
 BVC br7b ; backward
return7:

 LDA #$7F
 ADC #$7F ; set the overflow flag
 BVS br8a ; forward, no page crossing
br8b:
 BVS far8 ; forward, page crossing
br8a:
 BVC br8a ; not taken
 BVS br8b ; backward
return8:

implicit:
 INY
 DEY
 INX
 DEX
 NOP
 
accumulator:
 ASL
 ROL
 LSR
 ROR

jsrandmore:
 JSR trampoline ; more instructions tested at destination

stack:
 PHA
 PLA
 PHP
 PLP

flags:
 SEI
 CLI
 SED
 CLD

hopoverbackbranches:
 JMP continue

farbackbranches: ; reached with a page-crossing forward branch, returning the same way
far1:
 BPL return1
far2:
 BEQ return2
far3:
 BMI return3
far4:
 BNE return4
far5:
 BCC return5
far6:
 BCS return6
far7:
 BVC return7
far8:
 BVS return8

trampoline: ; testing a JSR but let's test an RTI too while we're here
 JSR rtitest
 ; we'll place an RTS and jump to it
 LDA returnop
 LDX #0
 STA (zdummy,X)
 JMP (zdummy)
returnop:
 RTS
 
rtitest:
 PLA
 CLC
 ADC #1 ; don't bother incrementing the high byte we should be safe
 PHA
 PHA
 RTI

continue:
 LDX #0 ; non-page-crossing
 BIT zdummy
 BIT dummy
 LDY zdummy
 STY zdummy
 LDY dummy
 STY dummy
 LDY zdummy,X
 STY zdummy,X
 LDY dummy,X
 DEX ; page crossing or wrapping
 LDY zdummy,X
 STY zdummy,X
 LDY dummy,X

 CPY #zdummy
 CPY zdummy
 CPY dummy
 CPX #zdummy
 CPX zdummy
 CPX dummy

end:
 .DB $bb ; pseudo-op to terminate Kowalski simulator
 
 JMP start
