FROM debian:bookworm

ARG OPENSSL_VERSION=3.3.1
ARG LIBJPEG_TURBO_VERSION=3.0.1
ARG X264_REF=stable

RUN apt-get update && apt-get install -y \
  build-essential git curl ca-certificates \
  autoconf automake libtool pkg-config \
  python3 cmake ninja-build perl \
  && rm -rf /var/lib/apt/lists/*

# Rust
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=/root/.cargo/bin:/opt/musl-cross/bin:$PATH

RUN apt-get update && apt-get install wget -y
# musl-cross-make
RUN git clone https://github.com/richfelker/musl-cross-make.git /tmp/mcm && \
  cd /tmp/mcm && \
  printf '%s\n' \
  'TARGET = arm-linux-musleabihf' \
  'OUTPUT = /opt/musl-cross' \
  'GCC_VER = 9.4.0' \
  'MUSL_VER = 1.2.5' \
  'LINUX_HEADERS_VER = 6.6.10' \
  > config.mak && \
  make -j$(nproc) && make install && \
  rm -rf /tmp/mcm

# Rust target
RUN rustup target add armv7-unknown-linux-musleabihf

# Build libusb for target
RUN git clone -b v1.0.27 https://github.com/libusb/libusb.git /tmp/libusb && \
  cd /tmp/libusb && \
  sed -i 's/use_udev=yes/use_udev=no/' configure.ac && \
  ./autogen.sh && set -x && \
  env \
  CC=arm-linux-musleabihf-gcc \
  CXX=arm-linux-musleabihf-g++ \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  SYSROOT=/opt/musl-cross/arm-linux-musleabihf \
  CFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf -O2 -fPIC -std=gnu11 -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  LDFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf" \
  ./configure \
  --disable-udev \
  --disable-examples-build \
  --build=x86_64-pc-linux-gnu \
  --host=arm-linux-musleabihf \
  --prefix=/usr && \
  make -j$(nproc) && \
  make DESTDIR=/opt/musl-cross/arm-linux-musleabihf install && \
  rm -rf /tmp/libusb

ENV SYSROOT=/opt/musl-cross/arm-linux-musleabihf
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_SYSROOT_DIR=$SYSROOT
ENV PKG_CONFIG_LIBDIR=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig
ENV PKG_CONFIG_PATH=$PKG_CONFIG_LIBDIR:/usr/lib/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig
ENV OPENSSL_DIR=$SYSROOT/usr
ENV OPENSSL_STATIC=1
ENV TURBOJPEG_STATIC=1
ENV X264_STATIC=1

RUN cd /tmp && \
  curl -fsSLO https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz && \
  tar -xzf openssl-${OPENSSL_VERSION}.tar.gz && \
  cd openssl-${OPENSSL_VERSION} && \
  CFLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  ./Configure linux-armv4 \
  no-shared \
  no-tests \
  no-dso \
  no-engine \
  --cross-compile-prefix=arm-linux-musleabihf- \
  --prefix=/usr \
  --openssldir=/etc/ssl \
  --libdir=lib && \
  make -j"$(nproc)" && \
  make DESTDIR="${SYSROOT}" install_sw && \
  rm -rf /tmp/openssl-${OPENSSL_VERSION} /tmp/openssl-${OPENSSL_VERSION}.tar.gz

RUN  printf '%s\n' \
  'deb http://deb.debian.org/debian bookworm main contrib non-free non-free-firmware' \
  'deb http://deb.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware' \
  'deb http://deb.debian.org/debian bookworm-updates main contrib non-free non-free-firmware' \
  > /etc/apt/sources.list
RUN  apt-get update && apt-get install -y libopus-dev libfdk-aac-dev
RUN apt-get update && apt-get install -y \
  ffmpeg \
  libavcodec-dev \
  libavformat-dev \
  libavutil-dev \
  libavdevice-dev \
  libavfilter-dev \
  libswscale-dev \
  libswresample-dev
RUN apt-get update && apt-get install -y \
  libsdl2-dev
RUN apt-get install libclang-dev -y

ENV BINDGEN_EXTRA_CLANG_ARGS="\
  --target=armv7-unknown-linux-musleabihf \
  --sysroot=/opt/musl-cross/arm-linux-musleabihf \
  -I/opt/musl-cross/arm-linux-musleabihf/usr/include"

RUN git clone https://github.com/xiph/opus.git /tmp/opus && \
  cd /tmp/opus && \
  ./autogen.sh && \
  env \
  SYSROOT=/opt/musl-cross/arm-linux-musleabihf \
  CC=arm-linux-musleabihf-gcc \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  CFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  LDFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf" \
  ./configure \
  --host=arm-linux-musleabihf \
  --prefix=/usr \
  --enable-static \
  --disable-shared \
  --disable-doc \
  --disable-extra-programs && \
  make -j$(nproc) && \
  make DESTDIR=/opt/musl-cross/arm-linux-musleabihf install && \
  rm -rf /tmp/opus

RUN git clone https://github.com/mstorsjo/fdk-aac.git /tmp/fdk-aac && \
  cd /tmp/fdk-aac && \
  autoreconf -fi && \
  env \
  SYSROOT=/opt/musl-cross/arm-linux-musleabihf \
  CC=arm-linux-musleabihf-gcc \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  CFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  LDFLAGS="--sysroot=/opt/musl-cross/arm-linux-musleabihf" \
  ./configure \
  --host=arm-linux-musleabihf \
  --prefix=/usr \
  --enable-static \
  --disable-shared \
  --disable-silent-rules && \
  make -j$(nproc) && \
  make DESTDIR=/opt/musl-cross/arm-linux-musleabihf install && \
  rm -rf /tmp/fdk-aac

RUN cd /tmp && \
  curl -fsSLO https://downloads.sourceforge.net/libjpeg-turbo/libjpeg-turbo-${LIBJPEG_TURBO_VERSION}.tar.gz && \
  tar -xzf libjpeg-turbo-${LIBJPEG_TURBO_VERSION}.tar.gz && \
  cmake -S libjpeg-turbo-${LIBJPEG_TURBO_VERSION} -B libjpeg-turbo-build \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_SYSTEM_NAME=Linux \
  -DCMAKE_SYSTEM_PROCESSOR=arm \
  -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY \
  -DCMAKE_FIND_ROOT_PATH=${SYSROOT} \
  -DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER \
  -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY \
  -DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY \
  -DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=ONLY \
  -DCMAKE_C_COMPILER=arm-linux-musleabihf-gcc \
  -DCMAKE_CXX_COMPILER=arm-linux-musleabihf-g++ \
  -DCMAKE_SYSROOT=${SYSROOT} \
  -DCMAKE_C_FLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  -DCMAKE_CXX_FLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  -DCMAKE_INSTALL_PREFIX=/usr \
  -DCMAKE_INSTALL_BINDIR=${SYSROOT}/usr/bin \
  -DCMAKE_INSTALL_LIBDIR=${SYSROOT}/usr/lib \
  -DCMAKE_INSTALL_INCLUDEDIR=${SYSROOT}/usr/include \
  -DCMAKE_INSTALL_DATAROOTDIR=${SYSROOT}/usr/share \
  -DCMAKE_INSTALL_MANDIR=${SYSROOT}/usr/share/man \
  -DCMAKE_INSTALL_DOCDIR=${SYSROOT}/usr/share/doc/libjpeg-turbo \
  -DWITH_TURBOJPEG=ON \
  -DENABLE_SHARED=OFF \
  -DENABLE_STATIC=ON \
  -DWITH_JPEG8=ON \
  && cmake --build libjpeg-turbo-build --parallel \
  && cmake --install libjpeg-turbo-build \
  && find ${SYSROOT}/usr -name '*turbojpeg*' -o -name '*.pc' \
  && mkdir -p ${SYSROOT}/usr/share/pkgconfig \
  && ls -la ${SYSROOT}/usr/lib ${SYSROOT}/usr/lib/pkgconfig ${SYSROOT}/usr/share/pkgconfig \
  && PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  pkg-config --modversion libturbojpeg \
  && rm -rf /tmp/libjpeg-turbo-${LIBJPEG_TURBO_VERSION}.tar.gz /tmp/libjpeg-turbo-${LIBJPEG_TURBO_VERSION} /tmp/libjpeg-turbo-build

RUN git clone --depth 1 --branch ${X264_REF} https://code.videolan.org/videolan/x264.git /tmp/x264 && \
  cd /tmp/x264 && \
  CC=arm-linux-musleabihf-gcc \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  ./configure \
  --host=arm-linux \
  --cross-prefix=arm-linux-musleabihf- \
  --sysroot=${SYSROOT} \
  --prefix=/usr \
  --libdir=/usr/lib \
  --includedir=/usr/include \
  --enable-static \
  --disable-cli \
  --disable-opencl \
  --enable-pic \
  --extra-cflags="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  --extra-ldflags="--sysroot=${SYSROOT}" && \
  make -j"$(nproc)" && \
  make DESTDIR=${SYSROOT} install-lib-static install-lib-dev && \
  PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  pkg-config --modversion x264 && \
  rm -rf /tmp/x264

ARG PIXMAN_VERSION=0.43.4
ARG DBUS_VERSION=1.14.10
ARG EXPAT_VERSION=2.6.4
ARG EXPAT_RELEASE_TAG=R_2_6_4
ARG LIBYUV_REF=main

RUN apt-get update && apt-get install -y meson xz-utils && \
  rm -rf /var/lib/apt/lists/*

ENV DBUS_1_STATIC=1
ENV PIXMAN_1_STATIC=1
ENV YUV_STATIC=1

RUN git clone --depth 1 --branch ${LIBYUV_REF} https://chromium.googlesource.com/libyuv/libyuv /tmp/libyuv && \
  cmake -S /tmp/libyuv -B /tmp/libyuv-build \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_SYSTEM_NAME=Linux \
  -DCMAKE_SYSTEM_PROCESSOR=arm \
  -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY \
  -DCMAKE_FIND_ROOT_PATH=${SYSROOT} \
  -DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER \
  -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY \
  -DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY \
  -DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=ONLY \
  -DCMAKE_C_COMPILER=arm-linux-musleabihf-gcc \
  -DCMAKE_CXX_COMPILER=arm-linux-musleabihf-g++ \
  -DCMAKE_SYSROOT=${SYSROOT} \
  -DCMAKE_C_FLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  -DCMAKE_CXX_FLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  -DBUILD_SHARED_LIBS=OFF && \
  cmake --build /tmp/libyuv-build --parallel && \
  mkdir -p ${SYSROOT}/usr/lib ${SYSROOT}/usr/include && \
  cp "$(find /tmp/libyuv-build -name libyuv.a -print -quit)" ${SYSROOT}/usr/lib/libyuv.a && \
  cp /tmp/libyuv/include/libyuv.h ${SYSROOT}/usr/include/libyuv.h && \
  cp -R /tmp/libyuv/include/libyuv ${SYSROOT}/usr/include/ && \
  test -f ${SYSROOT}/usr/lib/libyuv.a && \
  test -f ${SYSROOT}/usr/include/libyuv.h && \
  rm -rf /tmp/libyuv /tmp/libyuv-build

RUN cd /tmp && \
  curl -fsSLO https://www.cairographics.org/releases/pixman-${PIXMAN_VERSION}.tar.gz && \
  tar -xzf pixman-${PIXMAN_VERSION}.tar.gz && \
  printf '%s\n' \
  '[binaries]' \
  "c = 'arm-linux-musleabihf-gcc'" \
  "ar = 'arm-linux-musleabihf-ar'" \
  "strip = 'arm-linux-musleabihf-strip'" \
  "pkg-config = 'pkg-config'" \
  '' \
  '[host_machine]' \
  "system = 'linux'" \
  "cpu_family = 'arm'" \
  "cpu = 'armv7'" \
  "endian = 'little'" \
  '' \
  '[properties]' \
  "sys_root = '${SYSROOT}'" \
  "pkg_config_libdir = '${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig'" \
  '' \
  '[built-in options]' \
  "c_args = ['--sysroot=${SYSROOT}', '-O2', '-fPIC', '-march=armv7-a', '-mfpu=neon', '-mfloat-abi=hard']" \
  "c_link_args = ['--sysroot=${SYSROOT}']" \
  > pixman-cross.ini && \
  meson setup pixman-build pixman-${PIXMAN_VERSION} \
  --cross-file pixman-cross.ini \
  --prefix=/usr \
  --libdir=lib \
  --includedir=include \
  -Ddefault_library=static \
  -Darm-simd=disabled \
  -Dneon=enabled && \
  meson compile -C pixman-build && \
  DESTDIR=${SYSROOT} meson install -C pixman-build && \
  PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  pkg-config --modversion pixman-1 && \
  test -f ${SYSROOT}/usr/lib/libpixman-1.a && \
  rm -rf /tmp/pixman-${PIXMAN_VERSION}.tar.gz /tmp/pixman-${PIXMAN_VERSION} /tmp/pixman-build /tmp/pixman-cross.ini

RUN cd /tmp && \
  curl -fsSLO https://github.com/libexpat/libexpat/releases/download/${EXPAT_RELEASE_TAG}/expat-${EXPAT_VERSION}.tar.xz && \
  tar -xJf expat-${EXPAT_VERSION}.tar.xz && \
  cd expat-${EXPAT_VERSION} && \
  CC=arm-linux-musleabihf-gcc \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  CFLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  LDFLAGS="--sysroot=${SYSROOT}" \
  ./configure \
  --host=arm-linux-musleabihf \
  --prefix=/usr \
  --disable-shared \
  --enable-static \
  --without-docbook && \
  make -j"$(nproc)" && \
  make DESTDIR=${SYSROOT} install && \
  PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  pkg-config --modversion expat && \
  test -f ${SYSROOT}/usr/lib/libexpat.a && \
  rm -rf /tmp/expat-${EXPAT_VERSION}.tar.xz /tmp/expat-${EXPAT_VERSION}

RUN cd /tmp && \
  curl -fsSLO https://dbus.freedesktop.org/releases/dbus/dbus-${DBUS_VERSION}.tar.xz && \
  tar -xJf dbus-${DBUS_VERSION}.tar.xz && \
  cd dbus-${DBUS_VERSION} && \
  PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  CC=arm-linux-musleabihf-gcc \
  AR=arm-linux-musleabihf-ar \
  RANLIB=arm-linux-musleabihf-ranlib \
  STRIP=arm-linux-musleabihf-strip \
  CFLAGS="--sysroot=${SYSROOT} -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard" \
  LDFLAGS="--sysroot=${SYSROOT}" \
  ./configure \
  --host=arm-linux-musleabihf \
  --prefix=/usr \
  --disable-shared \
  --enable-static \
  --disable-tests \
  --disable-asserts \
  --disable-checks \
  --disable-xml-docs \
  --disable-doxygen-docs \
  --disable-systemd \
  --disable-selinux \
  --disable-apparmor \
  --disable-libaudit \
  --without-x && \
  make -j"$(nproc)" && \
  make DESTDIR=${SYSROOT} install && \
  PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig \
  PKG_CONFIG_SYSROOT_DIR=${SYSROOT} \
  pkg-config --modversion dbus-1 && \
  test -f ${SYSROOT}/usr/lib/libdbus-1.a && \
  rm -rf /tmp/dbus-${DBUS_VERSION}.tar.xz /tmp/dbus-${DBUS_VERSION}


ENV CC_armv7_unknown_linux_musleabihf=arm-linux-musleabihf-gcc
ENV CXX_armv7_unknown_linux_musleabihf=arm-linux-musleabihf-g++
ENV AR_armv7_unknown_linux_musleabihf=arm-linux-musleabihf-ar
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-musleabihf-gcc
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUSTFLAGS="-C link-arg=-lgcc"
ENV CFLAGS_armv7_unknown_linux_musleabihf="--sysroot=/opt/musl-cross/arm-linux-musleabihf -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard"
ENV CXXFLAGS_armv7_unknown_linux_musleabihf="--sysroot=/opt/musl-cross/arm-linux-musleabihf -O2 -fPIC -march=armv7-a -mfpu=neon -mfloat-abi=hard"

#ADD . /w
#RUN cd /w/catplay_c2a && cargo build --target armv7-unknown-linux-musleabihf
