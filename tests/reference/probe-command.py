"""Print the compiler command for a probe built against the frozen reference.

A probe that needs file-private reference behavior includes a frozen source
file instead of linking its object. Compiling that probe by hand would mean
maintaining a second copy of the reference build flags, so this script reads
them back from the reference build: the compiler and its flags come from
``compile_commands.json`` and the objects and libraries come from the recorded
link command of the reference executable. The probe therefore sees the same
compiler, the same conditional compilation, and the same libraries as the frozen
binary.

The printed command keeps the relative paths of the reference build, so it has
to run with the reference build directory as the working directory. Give the
probe source and the output file as absolute paths.
"""

import json
import shlex
import subprocess
import sys

EXECUTABLE = "terminal/xfce4-terminal"


def compile_command(build_dir, included_source):
    """Return the compiler and flags the reference build used for one source.

    The compiler itself comes from the recorded command rather than being
    assumed, because a reference built with a different compiler may not accept
    the flags that build recorded.
    """
    with open(f"{build_dir}/compile_commands.json", encoding="utf-8") as database:
        entries = json.load(database)

    matches = [
        entry for entry in entries if entry["file"].endswith(f"/{included_source}")
    ]
    if not matches:
        raise SystemExit(f"no compile command for {included_source}")
    if len(matches) > 1:
        raise SystemExit(
            f"{len(matches)} compile commands for {included_source}; "
            "the probe cannot tell which build options are the right ones"
        )

    recorded = shlex.split(matches[0]["command"])
    flags = []
    drop_next = False
    for argument in recorded[1:]:
        if drop_next:
            drop_next = False
        elif argument in ("-MQ", "-MF", "-o"):
            drop_next = True
        elif argument not in ("-MD", "-c") and not argument.endswith(included_source):
            flags.append(argument)
    return recorded[:1] + flags


def link_arguments(build_dir, excluded_objects):
    """Return the objects and libraries the reference executable links."""
    commands = subprocess.run(
        ["ninja", "-C", build_dir, "-t", "commands", EXECUTABLE],
        capture_output=True,
        check=True,
        encoding="utf-8",
    ).stdout.splitlines()

    arguments = []
    drop_next = False
    for argument in shlex.split(commands[-1])[1:]:
        if drop_next:
            drop_next = False
        elif argument == "-o":
            drop_next = True
        elif not any(argument.endswith(f"/{name}") for name in excluded_objects):
            arguments.append(argument)
    return arguments


def main(arguments):
    if len(arguments) < 4:
        raise SystemExit(
            "usage: probe-command.py BUILD_DIR INCLUDED_SOURCE PROBE_SOURCE "
            "OUTPUT [EXTRA_FLAG...]"
        )

    build_dir, included_source, probe_source, output = arguments[:4]
    excluded = ["main.c.o", f"{included_source}.o"]
    command = (
        compile_command(build_dir, included_source)
        + list(arguments[4:])
        + ["-o", output, probe_source]
        + link_arguments(build_dir, excluded)
    )
    print(shlex.join(command))


if __name__ == "__main__":
    main(sys.argv[1:])
