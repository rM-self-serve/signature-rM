# Signature-rM

### Inspired by and compatible with
https://github.com/Barabazs/rM-signature-patch

---

This simple program will remove the signature from the bottom of emails sent from the device. It will also take a backup before the modification, and allow reverting the modication from the backup or by performing the modification in reverse.


## ReMarkable Software Version Compatibility

- ✅ 1.9 - v3.8


### Type the following commands after ssh'ing into the ReMarkable Tablet

## Install

```
$ wget https://raw.githubusercontent.com/rM-self-serve/signature-rM/master/install-webint-ob.sh && bash install-webint-ob.sh
$ source ~/.bashrc
```

## Remove

```
$ wget https://raw.githubusercontent.com/rM-self-serve/signature-rM/master/remove-webint-ob.sh && bash remove-webint-ob.sh
```


## Usage

### To use Signature-rM, run:

```
$ systemctl stop xochitl
$ signature-rm apply
$ systemctl start xochitl
```
Or to revert the modification:
```
$ systemctl stop xochitl
$ signature-rm revert --backup  #or --reverse
$ systemctl start xochitl
```

## Before
![before](https://github.com/rM-self-serve/Signature-rM/assets/122753594/5191e05b-d0a2-4e33-9aeb-f8bf16c3f847)

## After
![after](https://github.com/rM-self-serve/Signature-rM/assets/122753594/7ccc84f3-9602-47bb-b6f1-dc794f6901ef)