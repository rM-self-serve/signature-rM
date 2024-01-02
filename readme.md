![Static Badge](https://img.shields.io/badge/reMarkable-v3.8-green)
![rM1: supported](https://img.shields.io/badge/rM1-supported-green)
![rM2: supported](https://img.shields.io/badge/rM2-supported-green)

# signature-rM

### Inspired by and compatible with
https://github.com/Barabazs/rM-signature-patch

---

This simple program will remove the signature from the bottom of emails sent from the device. It will also take a backup before the modification, and allow reverting the modication from the backup or by performing the modification in reverse.

This will only need to be installed once, but run everytime the device updates.

![demo](https://github.com/rM-self-serve/signature-rM/assets/122753594/59bb9621-af8c-4a33-a060-b0e0383053ba)


### Type the following commands after ssh'ing into the ReMarkable Tablet

## Install

**It is recommended to install via the [toltec package manager](https://toltec-dev.org/).** 

### With toltec

```
$ opkg update
$ opkg install signature-rm
$ opkg remove signature-rm
```

### No toltec

### Install
`$ wget https://raw.githubusercontent.com/rM-self-serve/signature-rM/master/install-signature-rm.sh && bash install-signature-rm.sh && source ~/.bashrc`


### Remove

`$ wget https://raw.githubusercontent.com/rM-self-serve/signature-rM/master/remove-signature-rm.sh && bash remove-signature-rm.sh`


## Usage

### To use signature-rM, run:

```
$ signature-rm apply
```
Or to revert the modification:
```
$ signature-rm revert --backup
# OR
$ signature-rm revert --reverse
```

## Before
![before](https://github.com/rM-self-serve/Signature-rM/assets/122753594/5191e05b-d0a2-4e33-9aeb-f8bf16c3f847)

## After
![after](https://github.com/rM-self-serve/Signature-rM/assets/122753594/7ccc84f3-9602-47bb-b6f1-dc794f6901ef)

## Manual install

You will need docker/podman, cargo, and the cargo crate named cross. There are other ways to cross compile for 32 bit arm as well.

`cross build --target armv7-unknown-linux-gnueabihf --release`

Then copy the binary 'target/armv7-unknown-linux-gnueabihf/release/signature-rm' to the device.
