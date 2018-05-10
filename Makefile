.PHONY: default build install uninstall test clean

default: build

build:
	jbuilder build src/graffophone.exe

test:
	jbuilder runtest -f

exec:
	jbuilder exec src/graffophone.exe

install:
	jbuilder install

uninstall:
	jbuilder uninstall

clean:
	jbuilder clean
# Optionally, remove all files/folders ignored by git as defined
# in .gitignore (-X).
#git clean -dfXq
