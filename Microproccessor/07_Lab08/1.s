  AREA    myData, DATA, READONLY
String  DCB    "TazKiaMalik", 0   

        AREA    myDataResult, DATA, READWRITE
length    DCD     0      ;this string has length 11 will show B in R0         


        AREA    myCode, CODE, READONLY
        ENTRY
        EXPORT main
main
        LDR     R0, =String     ;  R0 = pointer to start of the string
        BL      string_length     ; Calling function to calculate string
        LDR     R1, =length       ; R1 = address to store the result
        STR     R0, [R1]          ; Store result in memory(will find the value in R0)

STOP    B       STOP              ; Infinite loop to end program


string_length
        PUSH    {R1, R2}          ; Save registers
        MOV     R1, R0            ; R1 will point to current char
        MOV     R2, #0            ; R2 = length counter

loop
        LDRB    R0, [R1], #1     
        CMP     R0, #0            ; Check if it’s null terminator
        BEQ     done
        ADD     R2, R2, #1
        B       loop

done
        MOV     R0, R2            ; Move final count to R0
        POP     {R1, R2}          ;popping the registers used from stack
        BX      LR                ;moving back to main routine from subroutine

     END

