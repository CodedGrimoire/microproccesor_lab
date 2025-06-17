        AREA |.rodata|, DATA, READONLY

; -------- 32-bit variables --------
X     DCW     0x2012      
Y     DCW     0x30F0     
W     DCW     0x44F8      


; -------- Results for 32-bit logical operations --------
T1        DCD     0              
T2        DCD     0               
T3        DCD     0               
Result    DCD     0               

;DCD for 32-bit variable

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        
       
        LDR     R0, =X
        LDR     R1, [R0]        ; R1 = P

        LDR     R0, =Y
        LDR     R2, [R0]        ; R2 = Q

        LDR     R0, =W
        LDR     R3, [R0]        ; R3 = R

        LSR     R4, R1, #9      ; Shift right by 4
        AND     R4, R4, #0x3F   ; Mask 6 bits

        LSR     R5, R2, #1      ; Shift right by 5
        AND     R5, R5, #0x3F   ; Mask 6 bits

        LSR     R6, R3, #5      ; Shift right by 3
        AND     R6, R6, #0x3F   ; Mask 6 bits

        ADD     R7, R4, R5      ;P+Q
        EOR     R7, R7, R6      ;XOR with R

        AND     R7, R7, #0x3E
        
        LDR     R0, =Result
        STR     R7, [R0]        ;the final result F   

      
        ; Infinite loop
stop    B stop

        END
