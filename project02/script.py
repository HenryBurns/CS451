#!/usr/bin/env python3
def main():
    f = open("numbers.txt", "w")
    for i in range(10000):
        f.write(str(i) + "\n")
    f.close()
main()
