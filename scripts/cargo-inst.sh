#!/bin/sh
# Linux/Unix
sudo cp target/release/rax25kb /usr/local/bin/
sudo cp man/rax25kb.1 /usr/share/man/man1/
sudo cp examples/rax25kb.cfg /etc/rax25kb/rax25kb.cfg.example
sudo mandb

# Make working config
cp examples/rax25kb.cfg rax25kb.cfg
# Edit rax25kb.cfg for your setup

# Run
# rax25kb

