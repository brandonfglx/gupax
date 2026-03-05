#!/usr/bin/env bash

START_TIME=$EPOCHSECONDS

title() { printf "\n\e[1;93m%s\e[0m\n" "============================ $1 ============================"; }
check() {
	local CODE=$?
	if [[ $CODE = 0 ]]; then
		printf "${BASH_LINENO} | %s ... \e[1;92mOK\e[0m\n" "$1"
	else
		printf "${BASH_LINENO} | %s ... \e[1;91mFAIL\e[0m\n" "$1"
		exit $CODE
	fi
}
int() {
	exit 1
}

trap 'int' INT

title "Basic checks"
# Check for needed files
[[ -d skel ]]; check "skel"
[[ $1 = v* ]]; check "\$1 ... $1"
NEW_VER="$1"
cd skel; check "CD into skel"

# Check that [skel] directory contains everything
# and that the naming schemes are correct
title "Linux folder check"
[[ -f linux/gupax ]]; check "linux/gupax"
title "macOS-x64 folder check"
[[ -d macos-x64/Gupax.app ]]; check "macos-x64/Gupax.app"
title "macOS-arm64 folder check"
[[ -d macos-arm64/Gupax.app ]]; check "macos-arm64/Gupax.app"
title "Windows folder check"
[[ -f windows/Gupax.exe ]]; check "windows/Gupax.exe"

# Get random date for tar/zip
title "RNG Date"
RNG=$((EPOCHSECONDS-RANDOM*4)); check "RNG ... $RNG"
DATE=$(date -d @${RNG}); check "DATE ... $DATE"

title "Tar Linux"
# give execution permission
chmod +x linux/gupax
# Tar Linux Standalone
mv linux "gupax-$NEW_VER-linux-x64" 
tar -czpf "gupax-${NEW_VER}-linux-x64.tar.gz" "gupax-$NEW_VER-linux-x64" --owner=lm --group=lm ; check "tar linux"
# Remove dir
rm -r "gupax-$NEW_VER-linux-x64"; check "rm linux dir"

# x64
title "Tar macOS-x64"
# Tar macOS Standalone
mv macos-x64 "gupax-$NEW_VER-macos-x64"; check "macos-x64 -> gupax-$NEW_VER-macos-x64"
tar -czpf "gupax-${NEW_VER}-macos-x64.tar.gz" "gupax-$NEW_VER-macos-x64" --owner=lm --group=lm ; check "tar macos-x64"
# Remove dir
rm -r "gupax-$NEW_VER-macos-x64"; check "rm macos-x64 dir"

# ARM
title "Tar macOS-arm64"
# Tar macOS Standalone
mv macos-arm64 "gupax-$NEW_VER-macos-arm64"; check "macos-arm64 -> gupax-$NEW_VER-macos-arm64"
tar -czpf "gupax-${NEW_VER}-macos-arm64.tar.gz" "gupax-$NEW_VER-macos-arm64" --owner=lm --group=lm ; check "tar macos-arm64"
# Remove dir
rm -r "gupax-$NEW_VER-macos-arm64"; check "rm macos-arm64 dir"

title "Zip Windows"
# Zip Windows Standalone
mv windows "gupax-$NEW_VER-windows-x64"; check "windows -> gupax-$NEW_VER-windows-x64"
zip -qr "gupax-${NEW_VER}-windows-x64.zip" "gupax-$NEW_VER-windows-x64"; check "zip windows"
# Remove dir
rm -r "gupax-$NEW_VER-windows-x64"; check "rm windows dir"

# SHA256SUMS + Sign
title "Hash + Sign"
SHA256SUMS=$(sha256sum gupax* | gpg --clearsign --local-user 8EFFE4A8C0FD4B6D21C3AAB2EC6E5BB401C6362D); check "Hash + Sign"
echo "${SHA256SUMS}" > SHA256SUMS; check "Create SHA256SUMS file"
sha256sum -c SHA256SUMS; check "Verify SHA"
gpg --verify SHA256SUMS; check "Verify GPG"

# Get changelog + SHA256SUMS into clipboard
title "Clipboard"
clipboard() {
	echo "## SHA256SUM & [PGP Signature](https://github.com/cyrix126/gupax/blob/main/pgp/cyrix126.asc)"
	echo '```'
	cat SHA256SUMS
	echo '```'
}
CHANGELOG=$(clipboard); check "Create changelog + sign"
echo "$CHANGELOG" | wl-copy  $clipboard
check "Changelog into clipboard"

# Reset timezone
title "End"
printf "\n%s\n" "package.sh ... Took [$((EPOCHSECONDS-START_TIME))] seconds ... OK!"
