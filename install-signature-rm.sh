#!/usr/bin/env bash
# Copyright (c) 2023 rM-self-serve
# SPDX-License-Identifier: MIT

sigrm_sha256sum='eb82b757586b187edfd7ba742149b0929d96c124e488fef191524e13ae6bf63e'

release='v1.0.2'

installfile='./install-signature-rm.sh'
gh_pkgname='signature-rM'
pkgname='signature-rm'
localbin='/home/root/.local/bin'
binfile="${localbin}/${pkgname}"

wget_path=/home/root/.local/share/rM-self-serve/wget
wget_remote=http://toltec-dev.org/thirdparty/bin/wget-v1.21.1-1
wget_checksum=c258140f059d16d24503c62c1fdf747ca843fe4ba8fcd464a6e6bda8c3bbb6b5


remove_installfile() {
	read -r -p "Would you like to remove installation script? [y/N] " response
	case "$response" in
	[yY][eE][sS] | [yY])
		printf "Exiting installer and removing script\n"
		[[ -f $installfile ]] && rm $installfile
		;;
	*)
		printf "Exiting installer and leaving script\n"
		;;
	esac
}

echo "${gh_pkgname} ${release}"
echo "This program will remove the signature from the bottom"
echo "of emails sent from the device."
echo ''
echo "This program will be installed in ${localbin}"
echo "${localbin} will be added to the path in ~/.bashrc if necessary"
echo ''
read -r -p "Would you like to continue with installation? [y/N] " response
case "$response" in
[yY][eE][sS] | [yY])
	echo "Installing ${gh_pkgname}"
	;;
*)
	remove_installfile
	exit
	;;
esac

if [ -f "$wget_path" ] && ! sha256sum -c <(echo "$wget_checksum  $wget_path") > /dev/null 2>&1; then
    rm "$wget_path"
fi
if ! [ -f "$wget_path" ]; then
    echo "Fetching secure wget"
    # Download and compare to hash
    mkdir -p "$(dirname "$wget_path")"
    if ! wget -q "$wget_remote" --output-document "$wget_path"; then
        echo "Error: Could not fetch wget, make sure you have a stable Wi-Fi connection"
        exit 1
    fi
fi
if ! sha256sum -c <(echo "$wget_checksum  $wget_path") > /dev/null 2>&1; then
    echo "Error: Invalid checksum for the local wget binary"
    exit 1
fi
chmod 755 "$wget_path"

mkdir -p $localbin

case :$PATH: in
*:$localbin:*) ;;
*) echo "PATH=\"${localbin}:\$PATH\"" >>/home/root/.bashrc ;;
esac

pkg_sha_check() {
	if sha256sum -c <(echo "$sigrm_sha256sum  $binfile") >/dev/null 2>&1; then
		return 0
	else
		return 1
	fi
}

sha_fail() {
	echo "sha256sum did not pass, error downloading ${gh_pkgname}"
	echo "Exiting installer and removing installed files"
	[[ -f $binfile ]] && rm $binfile
	remove_installfile
	exit
}

[[ -f $binfile ]] && rm $binfile
"$wget_path" "https://github.com/rM-self-serve/${gh_pkgname}/releases/download/${release}/${pkgname}" \
	-O "$binfile"

if ! pkg_sha_check; then
	sha_fail
fi

chmod +x $binfile

echo ""
echo "Finished installing ${gh_pkgname}"
echo ""
echo "To use ${gh_pkgname}, run:"
echo "$ $pkgname apply"
echo ""

remove_installfile
