 #!/bin/bash -e

while true; do
    sar -b 1 5 > logs/ram.log
    sleep 1
done