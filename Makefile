CC = g++
CFLAGS = -std=c++11
PREFIX = /usr/local

yabl : main.o lexer.o parser.o
		$(CC) $(CFLAGS) -o build/yabl build/parts/main.o build/parts/lexer.o build/parts/parser.o

main.o : src/main.cpp
		$(CC) $(CFLAGS) -c src/main.cpp -o build/parts/main.o

lexer.o : src/lexer.cpp src/lexer.h
		$(CC) $(CFLAGS) -c src/lexer.cpp -o build/parts/lexer.o

parser.o : src/parser.cpp src/parser.h
		$(CC) $(CFLAGS) -c src/parser.cpp -o build/parts/parser.o

clean :
		-rm build/parts/*.o

install : yabl
		install -m 0755 build/yabl $(PREFIX)/bin