#!/bin/bash
exec 3<>/dev/tcp/127.0.0.1/17864
printf '%s\n' "$1" >&3
exec 3<&-
