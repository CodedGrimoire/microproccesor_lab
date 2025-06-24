        AREA    myCode,CODE,READONLY
        ENTRY
        EXPORT __main
__main

        ; Initialize the base address of the 2D array (Buffer)
        LDR   R5,=Buffer    ; R5 = Address of the start of the array

        ; Define the row and column we want to access
        LDR   R2,=RowIndex  ; R2 = Row Index
        LDR   R3,=ColIndex  ; R3 = Column Index

        ; Calculate offset for row-major form
        LDR   R6,=NumColumns  ; Load number of columns (4 in this case)
        MUL   R7, R2, R6      ; Multiply Row Index by number of columns
        ADD   R7, R7, R3      ; Add column index to the result (Row*Columns + Col)
        LSLS  R7, R7, #2      ; Multiply the result by 4 (size of each element)

        ; Calculate the address of the desired element
        ADD   R1, R5, R7      ; R1 = Address of the element (Buffer + offset)

        ; Load the element into R0 (to access it)
        LDR   R0, [R1]        ; R0 = Array[Row][Col]

        ; Store the element value in a memory location for display
        LDR   R4,=Result      ; Load the address where the result should be stored
        STR   R0, [R4]        ; Store the value of the accessed element in Result

STOP    B     STOP

        ALIGN

        ; Define the data for the 2D array (3x4 array)
        AREA    myData,DATA,READONLY
Buffer  DCD   1,2,3,4,   ; Row 0
        DCD   5,6,7,8,   ; Row 1
        DCD   9,10,11,12 ; Row 2

        ; Data for row and column index
        RowIndex  DCD 1      ; Access Row 1 (second row)
        ColIndex  DCD 2      ; Access Column 2 (third column)

        ; Define memory locations to store the result
        AREA    myDataResult,DATA,READWRITE
Result  DCD   0          ; To store the accessed value
        END
