#!/usr/bin/env bash

. ./cookie.env

shopt -s extglob

CURL="curl -H 'authority: adventofcode.com' -H 'dnt: 1' -H 'user-agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36' -H 'sec-fetch-user: ?1' -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3' -H 'sec-fetch-site: same-origin' -H 'sec-fetch-mode: navigate' -H 'accept-encoding: gzip, deflate, br' -H 'accept-language: nb,no;q=0.9,nn;q=0.8,en-US;q=0.7,en;q=0.6,de-DE;q=0.5,de;q=0.4' -H 'cookie: session=$AOC_SESSION' --compressed"

mkdir -p input/2022

# CURRENT_DAY=$(date +"%e")

# for i in $(seq -f "%02g" 1 25)
for i in $(seq -f "%02g" 1 $CURRENT_DAY)
do
  i_nopad=${i##+(0)}

  if [ ! -f "./input/2022/day$i.txt" ]; then
    echo "Getting day$i input..."
    echo $CURL "https://adventofcode.com/2022/day/$i_nopad/input" -o "./input/2022/day$i.txt" | bash
  fi
done

