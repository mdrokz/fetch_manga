for v in $(seq 1 $(ls "Dataset/$1/$2/Chapters" | wc -l))
do

echo $(ls "Dataset/$1/$2/Chapters/Chapter_$v" | wc -l)

done