# set -eux

### find all post url
# cat all-posts.html | htmlq '#the-list' '.view' -a href a | grep archives


### path update
cd static
for target in $(find . -name index.html); do
    p=$(dirname $target)
    sed -i "s|<img src=\".|<img src=\"|g" $target
done

# ### make root index.html
# cd static
# for target in $(exa -D -s name); do
#     echo "<a href=${target}>${target}</a>" >> index.html
# done
