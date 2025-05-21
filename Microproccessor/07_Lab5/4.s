        AREA |.rodata|, DATA, READONLY

; -------- 32-bit variables --------
X     DCB     0x00005F     ; X = input value for * bit



; -------- Results for 32-bit logical operations --------
T1        DCD     0              
T2        DCD     0               
T3        DCD     0               
Result    DCW     0               

;DCD for 32-bit variable

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        
       
        LDR     R0, =X
        LDRB    R1, [R0]        ; R1 = X

        AND     R2, R1, #0x0F   ;extracting low nibble
        AND     R3, R1, #0xF0   ;extracting high nibble
        LSR     R3, R3, #4      ; shifting right 4 bits
        LSL     R3, R3, #8      ;shifting it left 8 bit for a 16 bit word 
        ORR     R4, R3, R2      ;performing OR to add lsb to remaining 4 bit of the 16 bit word

       

       

       
     

        
        
        LDR     R0, =Result
        STRH     R4, [R0]        ;the final result 16 bit word

      
        ; Infinite loop
stop    B stop

        END
