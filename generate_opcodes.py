opcodes = """BRK impl
ORA X,ind
---
---
---
ORA zpg
ASL zpg
---
PHP impl
ORA #
ASL A
---
---
ORA abs
ASL abs
---
BPL rel
ORA ind,Y
---
---
---
ORA zpg,X
ASL zpg,X
---
CLC impl
ORA abs,Y
---
---
---
ORA abs,X
ASL abs,X
---
JSR abs
AND X,ind
---
---
BIT zpg
AND zpg
ROL zpg
---
PLP impl
AND #
ROL A
---
BIT abs
AND abs
ROL abs
---
BMI rel
AND ind,Y
---
---
---
AND zpg,X
ROL zpg,X
---
SEC impl
AND abs,Y
---
---
---
AND abs,X
ROL abs,X
---
RTI impl
EOR X,ind
---
---
---
EOR zpg
LSR zpg
---
PHA impl
EOR #
LSR A
---
JMP abs
EOR abs
LSR abs
---
BVC rel
EOR ind,Y
---
---
---
EOR zpg,X
LSR zpg,X
---
CLI impl
EOR abs,Y
---
---
---
EOR abs,X
LSR abs,X
---
RTS impl
ADC X,ind
---
---
---
ADC zpg
ROR zpg
---
PLA impl
ADC #
ROR A
---
JMP ind
ADC abs
ROR abs
---
BVS rel
ADC ind,Y
---
---
---
ADC zpg,X
ROR zpg,X
---
SEI impl
ADC abs,Y
---
---
---
ADC abs,X
ROR abs,X
---
---
STA X,ind
---
---
STY zpg
STA zpg
STX zpg
---
DEY impl
---
TXA impl
---
STY abs
STA abs
STX abs
---
BCC rel
STA ind,Y
---
---
STY zpg,X
STA zpg,X
STX zpg,Y
---
TYA impl
STA abs,Y
TXS impl
---
---
STA abs,X
---
---
LDY #
LDA X,ind
LDX #
---
LDY zpg
LDA zpg
LDX zpg
---
TAY impl
LDA #
TAX impl
---
LDY abs
LDA abs
LDX abs
---
BCS rel
LDA ind,Y
---
---
LDY zpg,X
LDA zpg,X
LDX zpg,Y
---
CLV impl
LDA abs,Y
TSX impl
---
LDY abs,X
LDA abs,X
LDX abs,Y
---
CPY #
CMP X,ind
---
---
CPY zpg
CMP zpg
DEC zpg
---
INY impl
CMP #
DEX impl
---
CPY abs
CMP abs
DEC abs
---
BNE rel
CMP ind,Y
---
---
---
CMP zpg,X
DEC zpg,X
---
CLD impl
CMP abs,Y
---
---
---
CMP abs,X
DEC abs,X
---
CPX #
SBC X,ind
---
---
CPX zpg
SBC zpg
INC zpg
---
INX impl
SBC #
NOP impl
---
CPX abs
SBC abs
INC abs
---
BEQ rel
SBC ind,Y
---
---
---
SBC zpg,X
INC zpg,X
---
SED impl
SBC abs,Y
---
---
---
SBC abs,X
INC abs,X
---"""

info = """immediate	ADC #oper	69	2	2  
zeropage	ADC oper	65	2	3  
zeropage,X	ADC oper,X	75	2	4  
absolute	ADC oper	6D	3	4  
absolute,X	ADC oper,X	7D	3	4* 
absolute,Y	ADC oper,Y	79	3	4* 
(indirect,X)	ADC (oper,X)	61	2	6  
(indirect),Y	ADC (oper),Y	71	2	5*
immediate	AND #oper	29	2	2  
zeropage	AND oper	25	2	3  
zeropage,X	AND oper,X	35	2	4  
absolute	AND oper	2D	3	4  
absolute,X	AND oper,X	3D	3	4* 
absolute,Y	AND oper,Y	39	3	4* 
(indirect,X)	AND (oper,X)	21	2	6  
(indirect),Y	AND (oper),Y	31	2	5*
accumulator	ASL A	0A	1	2  
zeropage	ASL oper	06	2	5  
zeropage,X	ASL oper,X	16	2	6  
absolute	ASL oper	0E	3	6  
absolute,X	ASL oper,X	1E	3	7
relative	BCC oper	90	2	2**
relative	BCS oper	B0	2	2**
relative	BEQ oper	F0	2	2**
zeropage	BIT oper	24	2	3  
absolute	BIT oper	2C	3	4
relative	BMI oper	30	2	2**
relative	BNE oper	D0	2	2**
relative	BPL oper	10	2	2**
implied	BRK	00	1	7
relative	BVC oper	50	2	2**
relative	BVS oper	70	2	2**
implied	CLC	18	1	2
implied	CLD	D8	1	2
implied	CLI	58	1	2
implied	CLV	B8	1	2
immediate	CMP #oper	C9	2	2  
zeropage	CMP oper	C5	2	3  
zeropage,X	CMP oper,X	D5	2	4  
absolute	CMP oper	CD	3	4  
absolute,X	CMP oper,X	DD	3	4* 
absolute,Y	CMP oper,Y	D9	3	4* 
(indirect,X)	CMP (oper,X)	C1	2	6  
(indirect),Y	CMP (oper),Y	D1	2	5*
immediate	CPX #oper	E0	2	2  
zeropage	CPX oper	E4	2	3  
absolute	CPX oper	EC	3	4
immediate	CPY #oper	C0	2	2  
zeropage	CPY oper	C4	2	3  
absolute	CPY oper	CC	3	4
zeropage	DEC oper	C6	2	5  
zeropage,X	DEC oper,X	D6	2	6  
absolute	DEC oper	CE	3	6  
absolute,X	DEC oper,X	DE	3	7
implied	DEX	CA	1	2
implied	DEY	88	1	2
immediate	EOR #oper	49	2	2  
zeropage	EOR oper	45	2	3  
zeropage,X	EOR oper,X	55	2	4  
absolute	EOR oper	4D	3	4  
absolute,X	EOR oper,X	5D	3	4* 
absolute,Y	EOR oper,Y	59	3	4* 
(indirect,X)	EOR (oper,X)	41	2	6  
(indirect),Y	EOR (oper),Y	51	2	5*
zeropage	INC oper	E6	2	5  
zeropage,X	INC oper,X	F6	2	6  
absolute	INC oper	EE	3	6  
absolute,X	INC oper,X	FE	3	7
implied	INX	E8	1	2
implied	INY	C8	1	2
absolute	JMP oper	4C	3	3  
indirect	JMP (oper)	6C	3	5
absolute	JSR oper	20	3	6
immediate	LDA #oper	A9	2	2  
zeropage	LDA oper	A5	2	3  
zeropage,X	LDA oper,X	B5	2	4  
absolute	LDA oper	AD	3	4  
absolute,X	LDA oper,X	BD	3	4* 
absolute,Y	LDA oper,Y	B9	3	4* 
(indirect,X)	LDA (oper,X)	A1	2	6  
(indirect),Y	LDA (oper),Y	B1	2	5*
immediate	LDX #oper	A2	2	2  
zeropage	LDX oper	A6	2	3  
zeropage,Y	LDX oper,Y	B6	2	4  
absolute	LDX oper	AE	3	4  
absolute,Y	LDX oper,Y	BE	3	4*
immediate	LDY #oper	A0	2	2  
zeropage	LDY oper	A4	2	3  
zeropage,X	LDY oper,X	B4	2	4  
absolute	LDY oper	AC	3	4  
absolute,X	LDY oper,X	BC	3	4*
accumulator	LSR A	4A	1	2  
zeropage	LSR oper	46	2	5  
zeropage,X	LSR oper,X	56	2	6  
absolute	LSR oper	4E	3	6  
absolute,X	LSR oper,X	5E	3	7
implied	NOP	EA	1	2
immediate	ORA #oper	09	2	2  
zeropage	ORA oper	05	2	3  
zeropage,X	ORA oper,X	15	2	4  
absolute	ORA oper	0D	3	4  
absolute,X	ORA oper,X	1D	3	4* 
absolute,Y	ORA oper,Y	19	3	4* 
(indirect,X)	ORA (oper,X)	01	2	6  
(indirect),Y	ORA (oper),Y	11	2	5*
implied	PHA	48	1	3
implied	PHP	08	1	3
implied	PLA	68	1	4
implied	PLP	28	1	4
accumulator	ROL A	2A	1	2  
zeropage	ROL oper	26	2	5  
zeropage,X	ROL oper,X	36	2	6  
absolute	ROL oper	2E	3	6  
absolute,X	ROL oper,X	3E	3	7
accumulator	ROR A	6A	1	2  
zeropage	ROR oper	66	2	5  
zeropage,X	ROR oper,X	76	2	6  
absolute	ROR oper	6E	3	6  
absolute,X	ROR oper,X	7E	3	7
implied	RTI	40	1	6
implied	RTS	60	1	6
immediate	SBC #oper	E9	2	2  
zeropage	SBC oper	E5	2	3  
zeropage,X	SBC oper,X	F5	2	4  
absolute	SBC oper	ED	3	4  
absolute,X	SBC oper,X	FD	3	4* 
absolute,Y	SBC oper,Y	F9	3	4* 
(indirect,X)	SBC (oper,X)	E1	2	6  
(indirect),Y	SBC (oper),Y	F1	2	5*
implied	SEC	38	1	2
implied	SED	F8	1	2
implied	SEI	78	1	2
zeropage	STA oper	85	2	3  
zeropage,X	STA oper,X	95	2	4  
absolute	STA oper	8D	3	4  
absolute,X	STA oper,X	9D	3	5  
absolute,Y	STA oper,Y	99	3	5  
(indirect,X)	STA (oper,X)	81	2	6  
(indirect),Y	STA (oper),Y	91	2	6
zeropage	STX oper	86	2	3  
zeropage,Y	STX oper,Y	96	2	4  
absolute	STX oper	8E	3	4
zeropage	STY oper	84	2	3  
zeropage,X	STY oper,X	94	2	4  
absolute	STY oper	8C	3	4
implied	TAX	AA	1	2
implied	TAY	A8	1	2
implied	TSX	BA	1	2
implied	TXA	8A	1	2
implied	TXS	9A	1	2
implied	TYA	98	1	2  """

info = info.split("\n")

def create_bytes_cycles(info_list):
    new_info_list = {}
    for info in info_list:
        address_mode, name, opcode, byte_num, cycles = info.split("\t")

        opcode = f"0x{opcode.lower()}"

        cycles = cycles.replace(" ", "")

        extras = 0 if len(cycles) == 1 else len(cycles) - 1

        cycles = int(cycles[0])
        byte_num = int(byte_num)

        new_info_list[opcode] = (byte_num, cycles, extras)

    return new_info_list

info = create_bytes_cycles(info)

opcodes = opcodes.split("\n")

def convert_address_mode(address_mode):
    if (address_mode == "abs") or (address_mode == "absolute"):
        address_mode = "AddressingMode::Absolute"

    elif (address_mode == "abs,X") or (address_mode == "absolute,X"):
        address_mode = "AddressingMode::AbsoluteX"

    elif (address_mode == "abs,Y") or (address_mode == "absolute,Y"):
        address_mode = "AddressingMode::AbsoluteY"

    elif (address_mode == "#") or (address_mode == "immediate"):
        address_mode = "AddressingMode::Immediate"

    elif (address_mode == "impl") or (address_mode == "implied"):
        address_mode = "AddressingMode::NoneAddressing"

    elif (address_mode == "ind") or (address_mode == "indirect"):
        address_mode = "AddressingMode::Indirect"

    elif (address_mode == "X,ind") or (address_mode == "(indirect,X)"):
        address_mode = "AddressingMode::IndirectX"

    elif (address_mode == "ind,Y") or (address_mode == "(indirect),Y"):
        address_mode = "AddressingMode::IndirectY"

    elif (address_mode == "zpg") or (address_mode == "zeropage"):
        address_mode = "AddressingMode::ZeroPage"

    elif (address_mode == "zpg,X") or (address_mode == "zeropage,X"):
        address_mode = "AddressingMode::ZeroPageX"

    elif (address_mode == "zpg,Y") or (address_mode == "zeropage,Y"):
        address_mode = "AddressingMode::ZeroPageY"

    elif (address_mode == "rel") or (address_mode == "relative"):
        address_mode = "AddressingMode::Immediate"

    elif (address_mode == "A") or (address_mode == "accumulator"):
        address_mode = "AddressingMode::NoneAddressing"

    return address_mode

def create_opcodes(opcodes):
    new_opcodes = []

    for index, opcode in enumerate(opcodes):
        hex_value = hex(index)

        if len(hex_value) == 3:
            hex_value = hex_value.replace("x", "x0")

        if opcode != "---":
            name, address_mode = opcode.split(" ")

            address_mode = convert_address_mode(address_mode)

            byte_num, cycles, extras = info[hex_value]

            # new_opcodes.append((hex_value, name, address_mode))
            new_opcodes.append(f"codes.insert({hex_value}, OpCode::new(\"{name}\", {byte_num}, {cycles}, {address_mode})); // extras {extras}")

    return "\n".join(new_opcodes)

opcodes = create_opcodes(opcodes)

print(opcodes)