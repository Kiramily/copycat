# Copy Cat

Copy Cat is a multithreaded copy utility designed to quickly and efficiently copy files.

# Installation

_soon_

# Usage

`copycat <source directroy> <destination directory>`
The Program will create the destination Directory if it doesn't exists.

# Limitations

The program is best suited for copying many small files rather than large files, as it makes use of multiple threads to accelerate file copying operations, Copying multiple big Files at the same time would max out the HDD/SSD which slows the copying down. Also the Copying should be done from one Disk to another, and not on the same Disk.

# Bugs

Don't run it as root or admin, this created a folder without any permissions on Windows for me.

# License

Copy Cat is licensed under the MIT License. Please see the [LICENSE](/LICENSE) file for more details.
