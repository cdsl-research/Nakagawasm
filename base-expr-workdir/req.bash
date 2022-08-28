tokens=()

for i in {0..1024}; do
    sleep 1
    tokens+=($(curl -s localhost:8000/login))
done

for token in "${tokens[@]}"; do
    sleep 1
    curl -b "Authorization=${token}" -s localhost:8000/logout
    echo
done
