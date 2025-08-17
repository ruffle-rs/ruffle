#!/bin/bash

set -e

echo "1. Downloading NotoSans-Regular.ttf"
wget -nv https://github.com/notofonts/notofonts.github.io/raw/eb3fdd45f361e6ae8bb5e5ac75f90f7482d02c80/fonts/NotoSans/full/ttf/NotoSans-Regular.ttf

echo "2. Downloading NotoSansMath-Regular.ttf"
wget -nv https://github.com/notofonts/notofonts.github.io/raw/eb3fdd45f361e6ae8bb5e5ac75f90f7482d02c80/fonts/NotoSansMath/full/ttf/NotoSansMath-Regular.ttf

echo "3. Subsetting NotoSans-Regular.ttf"
pyftsubset --unicodes-file=unicodes-file.txt NotoSans-Regular.ttf

echo "4. Removing MATH layout from NotoSansMath-Regular.ttf"
# See https://github.com/ruffle-rs/ruffle/issues/20337#issuecomment-3039836137
# Remove OpenType math layout
pyftsubset NotoSansMath-Regular.ttf "*" --drop-tables+=MATH --output-file=NotoSansMath-Regular-NoTable.ttf

echo "5. Subsetting NotoSansMath-Regular-NoTable.ttf"
# 2200-22FF Mathematical Operators
pyftsubset --unicodes=2200-22FF NotoSansMath-Regular-NoTable.ttf

echo "6. Merging fonts"
pyftmerge NotoSans-Regular.subset.ttf NotoSansMath-Regular-NoTable.subset.ttf

echo "7. Zipping result"
# Pure gzip (no headers or other sections)
cat merged.ttf | gzip --best --no-name | tail --bytes=+11 | head --bytes=-8 > notosans.subset.ttf.gz

echo "DONE: Created notosans.subset.ttf.gz"

echo "8. Removing artifacts"
rm *.ttf
