from _thongna import load_dict as rust_load_dict # Import inside test to avoid issues
from _thongna import newmm as rust_newmm
from pathlib import Path
from typing import List, Tuple

def load_dict(file_path: str, dict_name: str) -> Tuple[str, bool]:
    """Load dictionary from a file.

    Load a dictionary file into an in-memory dictionary collection,
    and assigned dict_name to it.
    *** This function does not override an existing dict name. ***

    :param file_path: Path to a dictionary file
    :type file_path: str
    :param dict_name: A unique dictionary name, use for reference.
    :type dict_name: str
    :return tuple[human_readable_result_str, bool]
    """
    path = Path(file_path).resolve()

    return rust_load_dict(str(path), dict_name)


def newmm(
    text: str,
    dict_name: str,
    safe: bool = False,
    parallel: bool = False,
) -> List[str]:
    """Break text into tokens.

    This method is an implementation of newmm segmentaion.
    Support multithread mode - set by parallel flag.

    :param text: Input text
    :type text: str
    :param dict_name: Dictionary name, as assigned in load_dict()
    :type dict_name: str
    :param safe: Use safe mode to avoid long waiting time in
        a text with lots of ambiguous word boundaries,
        defaults to False
    :type safe: bool, optional
    :param parallel: Use multithread mode, defaults to False
    :type parallel: bool, optional
    :return: List of tokens
    :rtype: List[str]
    """
    if not text or not isinstance(text, str):
        return []

    result = rust_newmm(text, dict_name, safe, parallel)

    return result