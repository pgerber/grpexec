# grpexec: Switch Group and Execute Command

**WARNING:** This project solely exists because I wanted to test Rust's FFI binding. There are better ways to implement
this.  For instance by using existing, well tested, libraries. Do not use in production.

## Setup

Install the **grpexec** binary in your **$PATH**. Also, either a) set the user to root and set the SUID bit or b) set
the file capability **CAP_SETGID**. This is needed because a regular, unprivileged, users can't change the GID of
processes, not even those of its own processes.

## Usage

### Syntax

```
grpexec GROUP COMMAND [ARG]...
```

### Example:

Execute ``firefox -P`` in group ``torify``.

```
grpexec torify firefox -P
```

## Troubleshooting

* ``ERROR: Failed to change group to "torify": failed to set new GID: Operation not permitted``

  Check your SUID / file capability setup
