for i in $(seq 0 $(ls "Chapters/Chapter_$1" | wc -l))
do
    tesseract "Chapters/Chapter_$1/$i.png" result

    RESULT=$(awk '/STORY/ || /ART/ || /BY/ || /Chapter/' result.txt)

    FILE=$(cat result.txt)

    if [[ ${#FILE} -le 0 ]] 
    then
         rm "Chapters/Chapter_$1/$i.png"
    fi
    
    
    if [[ ${#RESULT} -gt 0 ]] 
    then
         rm "Chapters/Chapter_$1/$i.png"
    fi

done