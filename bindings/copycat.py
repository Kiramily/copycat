from ctypes import cdll
import os
from enum import Flag
import multiprocessing


class CopyFlags(Flag):
    FlagNone = 1 << 0
    Overwrite = 1 << 1
    Recursive = 1 << 2
    SkipExisting = 1 << 3
    NoOverwrite = 1 << 4
    FollowSymlinks = 1 << 5


if os.name == 'nt':
    __lib = cdll.LoadLibrary('copycat.dll')
else:
    __lib = cdll.LoadLibrary('./libcopycat.so')


def copy(source: str, destination: str, threads: int = multiprocessing.cpu_count(), flags: CopyFlags = CopyFlags.FlagNone):
    '''
        Copy a file or directory from source to destination.
        :param source: The source path. can be a file or directory.
        :param destination: The destination path. should be a directory.
        :param flags: The copy flags to use. See CopyFlags.
    '''
    print(source, destination, threads, flags.value)
    # return __lib.cc_copy(source, destination, flags, threads)
