# cuo
[![Build status](https://travis-ci.org/shockham/cuo.svg?branch=master)](https://travis-ci.org/shockham/cuo)

Tool to automate updating minor dependency versions in rust bin projects.

Loosely Based upon the following bash script:
```bash

#!/bin/bash

function updated_outdated {
    cargo update
    cargo outdated
    if [ $? -eq 0 ]
    then
        rg -q "Cargo.lock" .gitignore
        if [ $? -eq 1 ]
        then
            git add --all
            git commit -m "Update deps"
            if [ $? -eq 0 ]
            then
                git push origin master
            fi
        fi
    fi
}

find . -mindepth 1 -maxdepth 1 -type d | while read -r dir
do
    pushd $dir
    if [[ -f "Cargo.toml" ]]
    then
        echo "CHECKING $dir"
        updated_outdated
    fi
    popd
done

```
