# piex

`piex` is a command to execute a binary received from a pipe.

# How to use
```bash
echo -e '#include<stdio.h>\n int main() {printf("Hi, there\\n");}' | gcc -xc - -o /dev/stdout | piex
```

```bash
cat /no-execute/premission/binary | piex
```
