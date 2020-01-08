CC = g++
CFLAGS = -std=c++11
PREFIX = /usr/local

build/yabl : build/parts/main.o build/parts/lexer.o build/parts/parser.o build/parts/interpreter.o
		$(CC) $(CFLAGS) -o build/yabl build/parts/main.o build/parts/lexer.o build/parts/parser.o build/parts/interpreter.o

build/parts/main.o : src/main.cpp
		$(CC) $(CFLAGS) -c src/main.cpp -o build/parts/main.o

build/parts/lexer.o : src/lexer.cpp src/lexer.h
		$(CC) $(CFLAGS) -c src/lexer.cpp -o build/parts/lexer.o

build/parts/parser.o : src/parser.cpp src/parser.h
		$(CC) $(CFLAGS) -c src/parser.cpp -o build/parts/parser.o

build/parts/interpreter.o : src/interpreter.cpp src/interpreter.h
		$(CC) $(CFLAGS) -c src/interpreter.cpp -o build/parts/interpreter.o

clean :
		-rm build/parts/*.o

install : build/yabl
		install -m 0755 build/yabl $(PREFIX)/bin

uninstall : yabl
		sudo rm $(PREFIX)/bin/yabl