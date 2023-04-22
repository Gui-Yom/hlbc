set windows-shell := ["cmd", "/c"]

default:
    just --list

data file:
    just -d data --justfile data/justfile build {{file}}
