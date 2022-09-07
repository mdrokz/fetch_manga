for v in $(seq 1 $(ls "Dataset/$1/$2/Chapters" | wc -l))
do
for i in $(seq 0 $(ls "Dataset/$1/$2/Chapters/Chapter_$v" | wc -l))
do
        aws s3 cp "Dataset/$1/$2/Chapters/Chapter_$v/$i.png" "s3://machine-learning-objects/Dataset/$1/$2/Chapters/Chapter_$v/"
done

done