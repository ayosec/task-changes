#!/bin/bash

case "$1" in

  _show)
    cat <<'EOF'
color.warning=color9
column.padding=1
complete.all.tags=no
confirmation=yes
data.location=src/tests
date.iso=yes
dateformat=Y-M-D
dateformat.annotation=
dateformat.edit=Y-M-D H:N:S
EOF
    exit
    ;;

  export)
    cat <<'EOF'
[
{"id":1,"description":"Euripidis fabulis delectari","entry":"20210324T225518Z","modified":"20210324T225634Z","project":"first","status":"pending","tags":["next"],"uuid":"69352372-30bc-43bc-957b-2f2af36053f0","annotations":[{"entry":"20210324T225556Z","description":"http:\/\/example.com: \"data\""},{"entry":"20210324T225634Z","description":"a\nb"}],"urgency":17.7},
{"id":2,"description":"Quasi concordia","entry":"20210324T225526Z","modified":"20210324T225528Z","project":"second","status":"waiting","uuid":"e505f4ba-cb73-42a7-9301-a4b2c68533c9","wait":"20210403T225528Z","urgency":-2}
]
EOF
    ;;

  *)
    echo "Invalid $@"
    exit 1
    ;;

esac
