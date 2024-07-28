#!/bin/bash

cd $(dirname "$0")
rm -f ../tmp/paste_bin.db
mkdir -p ../tmp
sqlite3 ../tmp/paste_bin.db < user_db.sql
sqlite3 ../tmp/paste_bin.db < content_db.sql
