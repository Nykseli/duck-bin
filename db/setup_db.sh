#!/bin/bash

cd $(dirname "$0")
rm -f ../tmp/users.db
mkdir -p ../tmp
sqlite3 ../tmp/users.db < user_db.sql
