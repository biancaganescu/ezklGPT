 #!/bin/bash -e

while true; do
    sar -r 1 1 > logs/ram.log
    sleep 1
done