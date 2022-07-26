for v in $(seq 1 $(ls "$1" | wc -l))
do

echo $(ls "$1/Chapter_$v" | wc -l)

done