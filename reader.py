from struct import unpack
from typing import Optional


class Reader:
    def __init__(self, bytecode: bytes):
        self.bytecode: bytes = bytecode
        self.pos: int = 0

    def canRead(self, n: int) -> bool:
        return self.pos + n <= len(self.bytecode)

    def nextByte(self) -> int:
        if not self.canRead(1):
            raise IndexError(
                f"Attempted to read byte at position {self.pos}, but bytecode length is {len(self.bytecode)}"
            )
        value = self.bytecode[self.pos]
        self.pos += 1
        return value

    def nextChar(self) -> str:
        return chr(self.nextByte())

    def nextUint32(self) -> int:
        return self.unpackStruct(4, "Attempted to read 4 bytes at position ", "<I")

    def nextInt(self) -> int:
        # Note: nextInt and nextUint32 behave identically in Luau bytecode,
        # as instructions are 32-bit unsigned integers.
        b = [self.nextByte() for _ in range(4)]
        return (b[3] << 24) | (b[2] << 16) | (b[1] << 8) | b[0]

    def nextVarInt(self) -> int:
        result = 0
        shift = 0
        # FIX: Luau VarInts are strictly limited to 5 bytes (35 bits)
        for _ in range(5):
            if not self.canRead(1):
                raise IndexError(
                    f"Unexpected end of bytecode while reading VarInt at position {self.pos}"
                )
            b = self.nextByte()
            result |= (b & 0x7F) << shift
            if not (b & 0x80):
                break
            shift += 7
        else:
            raise ValueError(f"VarInt at position {self.pos} is too long (max 5 bytes)")
            
        return result

    def nextString(self) -> str:
        length = self.nextVarInt()
        if length < 0:
            raise ValueError(f"Invalid string length {length} at position {self.pos}")
        if not self.canRead(length):
            raise IndexError(
                f"Attempted to read string of length {length} at position {self.pos}, but bytecode length is {len(self.bytecode)}"
            )
        result = self.bytecode[self.pos : self.pos + length].decode("utf-8", errors="replace")
        self.pos += length
        return result

    def nextFloat(self) -> float:
        return self.unpackStruct(4, "Attempted to read float at position ", "<f")

    def nextDouble(self) -> float:
        return self.unpackStruct(8, "Attempted to read double at position ", "<d")

    def unpackStruct(self, n: int, error_message: str, fmt: str) -> Optional[int | float]:
        if not self.canRead(n):
            raise IndexError(
                f"{error_message}{self.pos}, but bytecode length is {len(self.bytecode)}"
            )
        value = unpack(fmt, self.bytecode[self.pos : self.pos + n])[0]
        self.pos += n
        return value

    def skip(self, n: int) -> None:
        if n < 0:
            raise ValueError("Cannot skip a negative number of bytes.")
        if not self.canRead(n):
            raise IndexError(
                f"Attempted to skip {n} bytes at position {self.pos}, but bytecode length is {len(self.bytecode)}"
            )
        self.pos += n

    def read(self, n: int) -> bytes:
        if n < 0:
            raise ValueError("Cannot read a negative number of bytes.")
        if not self.canRead(n):
            raise IndexError(
                f"Attempted to read {n} bytes at position {self.pos}, but bytecode length is {len(self.bytecode)}"
            )
        data = self.bytecode[self.pos : self.pos + n]
        self.pos += n
        return data
