#!/bin/bash
. ./.env

cd "$(dirname "$0")"
run () { echo '==>' $@; $@; }
warn () { local c=$?; echo '[WARN]' $@; return $c; }
fail () { local c=$?; echo '[FAIL]' $@; exit $c; }

run cross build --release --target=$TARGET \
  || fail "abort before upload due to failed compilation."

run ssh $ADMIN_REMOTE sudo systemctl disable bagel
run ssh $ADMIN_REMOTE sudo systemctl stop bagel

run ssh $REMOTE mkdir -p $DEST \
  && run scp target/$TARGET/release/bagel $REMOTE:${DEST}/bagel \
  && run scp .env.release $REMOTE:${dest}/.env \
  && run scp bagel.service $REMOTE:${dest}/bagel.service \
  || warn "could not upload all files."

run ssh $ADMIN_REMOTE sudo ln -s /home/rain/$DEST/bagel.service /etc/systemd/system/bagel.service \
  || warn "could not create systemd service symlink."

run ssh $ADMIN_REMOTE sudo systemctl start bagel
run ssh $ADMIN_REMOTE sudo systemctl enable bagel
