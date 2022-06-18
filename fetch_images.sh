for i in $(seq 0 $1)
do
  CHAPTER_FILE=$(cat chapters.json)
  CHAPTER=$(echo $CHAPTER_FILE | jq ".[$i].name" | tr -d \")

  CHAPTERS_LENGTH=$(echo $CHAPTER_FILE | jq ".[$i].images | length" | tr -d \")

  mkdir $2

  mkdir "$2/$CHAPTER"

  for im in $(seq 0 $CHAPTERS_LENGTH)
  do
    CHAPTER_LINK=$(echo $CHAPTER_FILE | jq ".[$i].images[$im]" | tr -d \")
    curl "$CHAPTER_LINK" > "$2/$CHAPTER/$im.png" &
  done
done