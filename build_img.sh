rm sdcard.img
dd if=/dev/zero of=sdcard.img bs=1M count=128
mkfs.vfat -F 32 -S 512 sdcard.img
mkdir -p mnt
sudo mount sdcard.img mnt
# 此处生成的是libc的测例
sudo cp -r ./testcases/oscomp ./mnt/
sudo umount mnt
rm -rf mnt
sudo chmod 777 sdcard.img