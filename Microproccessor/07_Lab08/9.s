        AREA |.rodata|, DATA, READONLY
Array          DCD     1,2,3,4,9,10,11         
ArrayLength    DCD     0x00000007


        AREA |.data|, DATA, READWRITE

; -------- Output variable --------
EvenCount      DCD     0               ; Storing the even count
OddCount       DCD     0               ; Storing the odd count


        AREA    myCode, CODE, READONLY
        ENTRY
        EXPORT  main

main
        LDR     R0, =Array        ; R0 = pointer to array
        LDR     R1, =ArrayLength  ; R1 = pointer to array length
        LDR     R1, [R1]          ; R1 = actual length value
        BL      odd_even          ; Calling the function

        LDR     R2, =EvenCount
        STR     R0, [R2]          ; Store even count

        LDR     R3, =OddCount
        STR     R1, [R3]          ; Store odd count

STOP    B       STOP              ; Infinite loop to stop execution


odd_even
        PUSH    {R2-R6}           ; Save working registers

        MOV     R2, #0            ; R2 = even counter
        MOV     R3, #0            ; R3 = odd counter
        MOV     R4, #0            ; R4 = index

loop_check
        CMP     R4, R1
        BGE     done_loop

        LDR     R5, [R0, R4, LSL #2] ; Load NumArray[R4]
        ANDS    R6, R5, #1            ; Test LSB (even/odd) performing and updating APSR
        BEQ     is_even         
        ADD     R3, R3, #1           ; Odd++
        B       next
is_even
        ADD     R2, R2, #1           ; Even++

next
        ADD     R4, R4, #1
        B       loop_check

done_loop
        MOV     R0, R2              ; R0 = even count
        MOV     R1, R3              ; R1 = odd count
        POP     {R2-R6}             ;popping the registers used from stack
        BX      LR                  ;moving back to main routine from subroutine

      

        END
