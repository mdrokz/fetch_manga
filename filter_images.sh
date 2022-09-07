for v in $(seq 1 $(ls "Dataset/$1/$2/Chapters" | wc -l))
do
for i in $(seq 0 $(ls "Dataset/$1/$2/Chapters/Chapter_$v" | wc -l))
do
    tesseract "Dataset/$1/$2/Chapters/Chapter_$v/$i.png" result

    RESULT=$(awk '/STORY/ || /ART/ || /BY/ || /Chapter/' result.txt)

    FILE=$(cat result.txt)

    if [[ ${#FILE} -le 0 ]] 
    then
         rm "Dataset/$1/$2/Chapters/Chapter_$v/$i.png"
    fi
    
    
    if [[ ${#RESULT} -gt 0 ]] 
    then
         rm "Dataset/$1/$2/Chapters/Chapter_$v/$i.png"
    fi

done

done