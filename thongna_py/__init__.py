from pathlib import Path
from typing import List, Tuple

from thongna import load_dict as rust_load_dict  # type: ignore
from thongna import newmm as rust_newmm  # type: ignore
from thongna import normalize as rust_normalize # type: ignore

def load_dict(file_path: str, dict_name: str) -> Tuple[str, bool]:
    """
    Load dictionary from a file.

    Load a dictionary file into an in-memory dictionary collection,
    and assign dict_name to it.
    This function does not override an existing dict name.

    Args:
        file_path (str): Path to a dictionary file
        dict_name (str): A unique dictionary name, used for reference

    Returns:
        Tuple[str, bool]: A tuple containing a human-readable result string and a boolean
    """
    path = Path(file_path).resolve()
    return rust_load_dict(str(path), dict_name)


def newmm(
    text: str,
    dict_name: str,
    safe: bool = False,
    parallel: bool = False,
) -> List[str]:
    """
    Break text into tokens.

    This method is an implementation of newmm segmentation.
    Supports multithread mode - set by parallel flag.

    Args:
        text (str): Input text
        dict_name (str): Dictionary name, as assigned in load_dict()
        safe (bool, optional): Use safe mode to avoid long waiting time in
            a text with lots of ambiguous word boundaries. Defaults to False.
        parallel (bool, optional): Use multithread mode. Defaults to False.

    Returns:
        List[str]: List of tokens
    """
    if not isinstance(text, str) or not text:
        return []

    return rust_newmm(text, dict_name, safe, parallel)

def normalize(text: str, whitespace_number: bool = True) -> str:
    """
    Normalize Thai text.

    This function normalizes Thai text by applying various rules to standardize
    the text representation.

    Args:
        text (str): Input text to be normalized
        whitespace_number (bool, optional): If True, adds spaces around numbers. 
                                            Defaults to True.

    Returns:
        str: Normalized text
    """
    return rust_normalize(text, whitespace_number)