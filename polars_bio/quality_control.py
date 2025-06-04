# Tu powinna być implementacja naszej funkcji
# chyba - jeszcze nie wiem dokładnie


from __future__ import annotations
__all__ = ["add_two_numbers_py"]
from polars_bio.polars_bio import add_two_numbers
from .io import read_fastq


def add_two_numbers_py(a, b):
    return add_two_numbers(a, b)

# def quality_control_py(sequence):
#     return quality_control(sequence)


