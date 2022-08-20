for i in $(seq 0 $1)
do
  CHAPTER_FILE=$(cat Dataset/$2/$3/chapters.json)
  CHAPTER=$(echo $CHAPTER_FILE | jq ".[$i].name" | tr -d \")

  CHAPTERS_LENGTH=$(echo $CHAPTER_FILE | jq ".[$i].images | length" | tr -d \")

  for im in $(seq 0 $CHAPTERS_LENGTH)
  do
    CHAPTER_LINK=$(echo $CHAPTER_FILE | jq ".[$i].images[$im]" | tr -d \")
    curl "$CHAPTER_LINK" > "Dataset/$2/$3/Chapters/$CHAPTER/$im.png"
  done
done