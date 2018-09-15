.PHONY: default build install uninstall test clean

default: build

build:
	dune build src/graffophone.exe

dbg:
	dune build src/graffophone.bc
	cp ./_build/default/src/graffophone.bc .

test:
	dune runtest -f

exec:
	dune exec src/graffophone.exe

install:
	dune install

uninstall:
	dune uninstall

clean:
	dune clean
# Optionally, remove all files/folders ignored by git as defined
# in .gitignore (-X).
#git clean -dfXq
