import re

REG = r"(.*?)_([a-zA-Z])"

def camel(match):
    return match.group(1) + match.group(2).upper()

def camel_upper(match):
    return match.group(1)[0].upper() + match.group(1)[1:] + match.group(2).upper()

f = open("lib/ssc/ast/old.h")
words = f.readlines()
f.close()
print(words)
results = [re.sub(REG, camel, w, 0) for w in words]
print(results)

f = open("lib/ssc/ast/lexer.h", "w")
for line in results:
    f.write(line)
f.close()
