set -eux

cat all-posts.html | htmlq '#the-list' '.view' -a href a | grep archives
