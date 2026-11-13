#!/usr/bin/env python3
"""Independent arithmetic checks for the E026 shortened fold-window boundary."""

F = (1 << 32) + 976


def fold_model(low: int, ext: int, nm: int, e: int, bits: int) -> tuple[int, int]:
    k = 3 + e
    low &= (1 << bits) - 1
    base = low + ext - (nm - ext) * F
    modulus = 1 << k
    m = (base * pow(F + 1, -1, modulus)) % modulus
    wtrue = base - m * F
    return wtrue, (wtrue % (1 << bits)) >> k


def check(bits: int, e: int, cases: list[tuple[int, int, int, int, bool]]) -> None:
    for low, ext, nm, expected, should_fail in cases:
        wtrue, _ = fold_model(low, ext, nm, e, bits)
        assert wtrue == expected, (bits, hex(low), wtrue, expected)
        fail = wtrue < 0 or wtrue >= (1 << bits)
        assert fail == should_fail, (bits, hex(low), wtrue, fail)


check(57, 5, [
    (0x20001030003DB6D, 2, 6, -1, True),
    (0x200000400000F3E, 2, 6, 0, False),
    (0x1FFFFF8FFFFE449, 7, 0, (1 << 57) - 256, False),
    (0x1FFFFF8FFFFE549, 7, 0, 1 << 57, True),
])

check(55, 3, [
    (0x8000430000FF6D, 2, 6, -1, True),
    (0x80000400000F3E, 2, 6, 0, False),
    (0x7FFFF8FFFFE509, 7, 0, (1 << 55) - 64, False),
    (0x7FFFF8FFFFE549, 7, 0, 1 << 55, True),
])

print("E026 boundary model: 8/8 pass; four in-range and four carry/borrow escapes")
