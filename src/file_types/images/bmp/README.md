# BMP (Bitmap) File Storage

## 参考

- [MSDN](https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage)
- [Wikipedia](https://en.wikipedia.org/wiki/BMP_file_format)

## 介绍

BMP 一种简单，古老但是现今仍然在大量使用的的一种（通常）无压缩的位图格式。由微软开发。

在色深在 8 位与以下时，BMP 使用索引方式进行存储。文件中必须包含调色板（Color Table），在图像数据（Image Data）中则存储着每一个像素对调色板的索引。

在色深为 16 位及以上的情况，BMP 使用直接色的方式进行存储。文件中不包含调色板，而在图像数据中则存有每个像素真实的颜色信息。

32 位的 BMP 可以保存 Alpha 通道的信息，但是常见的图像查看器似乎都不会读取，所以平时不太容易看到透明 BMP。

### 优点

- 格式简单，兼容性好，无需解压 / 解码，没有性能要求，门锁都能打开
- 直接存储像素信息，所以是无损格式

### 缺点

- 通常而言是一种无压缩格式，大图像对应大文件，不经济。

## 基础结构

```
┌──────────────────┐
│Bitmap File Header│ -> Required, BITMAPFILEHEADER
├──────────────────┤
│DIB Header        │ -> Required, BITMAPINFOHEADER(V3) / BITMAPV4HEADER / BITMAPV5HEADER
├──────────────────┤
│Color Table       │ -> Optional when color-depth <=8, RGBQUAD
├──────────────────┤
│Image Data        │
└──────────────────┘
```

### Bitmap File Header

必须。

### DIB Header

必须。

目前在使用的有三个版本：

- BITMAPINFOHEADER: 最常见的版本。
- BITMAPV4HEADER: 前者的扩展定义。增加了色彩空间类型（color space type）与 Gamma 纠正（gamma correction）信息。现在基本上见不到，有需要可以用 V5。
- BITMAPV5HEADER: 前者的扩展定义。增加了 ICC 配置文件（ICC color profiles）。其实也挺少见的，在一些工程场景（比如 PS 保存 BMP 勾选保存 ICC 的时候）会用到。

### Color Table

在 8 位色深及下必须，16 位及以上不应该存在。存储着调色板，格式极其简单。

### Image Data

必须。
存储着每一个像素的图像信息。
以行为包装单位，顺序为从下到上从左到右（即图像左下角为原点），顺序排列。每一行的数据需要补 0 到 4 的倍数位。
不同色深下每个像素存储的方式不同：

- 1 位色深：索引颜色，每个 bit 存储一个像素索引。每个字节存储 8 个像素，在一行的像素数量不是 8 的倍数时，最后的字节会在后面补 0。
- 4 位色深：索引颜色，每 2 个 bit 存储一个像素索引。每个字节存储 2 个像素。在一行的像素数量不是 4 的倍数时，最后的字节会在后面补 0。
- 8 位色深：索引颜色，每个字节存储 1 个像素，不需要像前面的一样压缩存储。
- 16 位色深：直接颜色，每 2 个字节存储 1 个像素。在 BI_RGB 模式下，RGB 各占 5 个 bit（RGB555），高位为红。最高位的 1 bit 置 0。
- 24 位色深：直接颜色，每 3 个字节存储 1 个像素。RGB 各占 1 个字节，存储顺序为 **BGR**。
- 32 位色深：直接颜色，每 4 个字节存储 1 个像素。在 BI_RGB 模式下，在 24 位的基础上增加了 Alpha 通道，占 1 个字节，存储顺序为 **BGRA**。

#### 掩码

为了更精确的表明在存储中哪些位对应哪个通道，在 16 位与 32 位色深中可以通过在 DIB Header 中将 BI_COMPRESSION 指定为 BI_BITFIELDS 来提供掩码，以明确每一个 bit 的含义。

```
 16 bit RGB565 Format
 Mask-R 1111100000000000 = 0x0000F800
 Mask-G 0000011111100000 = 0x000007E0
 Mask-B 0000000000011111 = 0x0000001F

        RRRRRGGGGGGBBBBB

Example 0011110000101100 = 0x3C2C (Range 0 ~ 0xFFFF)
      R 00111            = 0x0007 (Range 0 ~ 0x1F)
      G      100001      = 0x0021 (Range 0 ~ 0x3F)
      B            01100 = 0x000C (Range 0 ~ 0x1F)
```

在有掩码时，在 DIB Header BITMAPINFOHEADER 后需要紧跟着三个 DWORD（4 字节），分别用于表示 R、G、B 三色的掩码。
BITMAPV4HEADER 与 BITMAPV5HEADER 直接在 Header 定义中就提供了掩码对应的字段，即掩码就是 Header 的一部分。
