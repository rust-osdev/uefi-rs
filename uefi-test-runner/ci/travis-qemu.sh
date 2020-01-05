#!/bin/bash
# This script builds QEMU from source.

# Original taken from: https://github.com/jdub/travis-qemu
# Original license:
: '
Copyright (c) 2016 Jeff Waugh

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
'

set -e

VERSION=4.2.0
ARCHES=x86_64
TARGETS=${QEMU_TARGETS:=$(echo $ARCHES | sed 's#$# #;s#\([^ ]*\) #\1-softmmu \1-linux-user #g')}

if echo "$VERSION $TARGETS" | cmp --silent $HOME/qemu/.build -; then
  echo "qemu $VERSION up to date!"
  exit 0
fi

echo "VERSION: $VERSION"
echo "TARGETS: $TARGETS"

cd $HOME
rm -rf qemu

# Checking for a tarball before downloading makes testing easier :-)
test -f "qemu-$VERSION.tar.xz" || wget "https://download.qemu.org/qemu-$VERSION.tar.xz"
tar -xf "qemu-$VERSION.tar.xz"
cd "qemu-$VERSION"

./configure \
  --prefix="$HOME/qemu" \
  --target-list="$TARGETS" \
  --disable-docs \
  --disable-sdl \
  --disable-gtk \
  --disable-gnutls \
  --disable-gcrypt \
  --disable-nettle \
  --disable-curses \
  --static

make -j4
make install

echo "$VERSION $TARGETS" > $HOME/qemu/.build
