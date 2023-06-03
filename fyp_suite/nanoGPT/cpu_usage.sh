  #!/bin/bash -e
 while true; do sar -P ALL 1 1 > logs/uptime.log; sleep 1; done
