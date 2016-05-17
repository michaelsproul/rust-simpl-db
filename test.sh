#!/bin/bash

./gendata 100000 5 > data.txt
echo "Creating a table"
./create bigdata 5 1 "0,0:1,0:2,0:3,0:4,0"
echo "Inserting data"
time ./insert bigdata < data.txt
echo "Outputting data"
time ./select bigdata "?,?,?,?,?" > output.txt
sort data.txt > data.sorted
sort output.txt > output.sorted
./stats bigdata
echo "Comparing expected input to output"
diff -q data.sorted output.sorted
rm -f *.txt *.sorted bigdata.*
echo "Done!"
