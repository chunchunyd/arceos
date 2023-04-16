# Week 9

## 内核和应用程序合并编译

1. `makefile`调用`PHONY` ，通过`scripts/make/build.mk`的`_cargo_build`部分生成应用程序对应的bin文件。以`hello world`为例：

   `hello world`的`cargo.toml`依赖的是`libax = { path = "../../ulib/libax" }  `，因此会链接`libax`模块，此时`libax`模块的`cargo.toml`依赖了大量的内核模块，因此造成的效果便是内核和应用程序代码共同链接，生成了一个bin文件。

2. `makefile`调用汇编指令`call run_qemu`，通过`scripts/make/run_qemu`运行第一步生成的bin文件。由于该bin文件已经链接了内核和应用程序代码，所以可以直接运行。

## 与rcore区别

rcore的应用程序文件需要在`user`文件夹下和`lib`依此链接，生成多个bin文件。此时的bin文件无法单独在qemu上运行，需要内核进行加载（使用loader.rs直接链接或者引入文件系统）。



## 分离编译思路

指明`hello world`的`cargo.toml`依赖库为一个新的lib，如`user_lib`，该库的`cargo.toml`不依赖于内核，类似于rcore的lib。此时编译`hello world`生成`hello world.bin`之后再在内核中显式进行链接，链接方式可以参考`rcore`的`ch2`。

完成链接之后即可进行编译。

