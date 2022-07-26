for v in $(seq 1 $(ls "Chapters" | wc -l))
do
for i in $(seq 0 $(ls "Chapters/Chapter_$v" | wc -l))
do
    tesseract "Chapters/Chapter_$v/$i.png" result

    RESULT=$(awk '/STORY/ || /ART/ || /BY/ || /Chapter/' result.txt)

    FILE=$(cat result.txt)

    if [[ ${#FILE} -le 0 ]] 
    then
         rm "Chapters/Chapter_$v/$i.png"
    fi
    
    
    if [[ ${#RESULT} -gt 0 ]] 
    then
         rm "Chapters/Chapter_$v/$i.png"
    fi

done

done