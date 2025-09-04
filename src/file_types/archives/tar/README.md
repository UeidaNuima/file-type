# TAR

## 参考

- [GNU](https://www.gnu.org/software/tar/manual/html_node/Standard.html)
- [The Open Group - Pax](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pax.html)
- https://www.subspacefield.org/~vax/tar_format.html

## 介绍

tar 是一个打包 / 归档工具，同时也是一种格式。Tar 本身只是单纯把多个文件拼合成一个文件，不进行任何打包。最早 tar 是为了磁带存储设计的。

- 早期有多种不同的 tar 实现，它们之间不完全兼容。
- 在 1998 年，IEEE 通过 `POSIX IEEE P1003.1` 提供了 `UStar` 规范（Unix Standard Tar），统一了 tar 格式的标准。UStar 目前仍为主流格式，受到广泛的支持。
- 1997 年 Sun 提供了一种扩展格式，用于弥补 UStar 格式中的种种不足。后续该格式作为 POSIX.1-2001 标准提出，称为 tar 扩展（Extended Tar）或者 pax 格式。pax 格式完全兼容 USTar 标准，并且提出了通过虚拟的 tar entry 来扩展 USTar 元信息的能力。当前主流的 tar 工具都支持 pax，并且会尝试在原始 USTar 无法完全支持打包内容时主动使用 pax 格式进行扩展。

### 优点

- 格式简单，兼容性好，没有编解码 / 加解密的开销，门锁都能打开

### 缺点

- 文件没有顶头的 Magic Number，在没有扩展名时没法非常快速的判断是否为一个 tar 文件，现代 tar 只能尝试以 tar 结构来解析并且寻找 header 中的 `ustar` 魔法字段
- 由于结构简单所以抗攻击性弱，没有文件内容校验等能力，对损坏弱不禁风
- 早期 tar 工具存在一些安全性问题，比如接受绝对路径或者相对父路径的文件名，导致解压后会覆盖一些非预期的文件。在这种情况下由于可以通过恶意构造路径来损坏系统文件，该问题也被称为 tar 炸弹（Tarbomb）。
- 整个 tar 文件组成方式为一个顺序文件流，并且不包含 toc 等索引信息，因此在完整扫描整个文件之前无法进行随机访问。
- tar 规范没有规定在一个 tar 中出现多个同路径文件会发生什么事情，因此通常而言提取器的实现是依次提取，所以在 tar 文件中靠后出现的那个会覆盖靠前的，表现非常古怪

## 基础结构

```
               ┌─────────────┐
            ┌─ │Header Block │ ──► Required in one entry
Tar Entry ◄─┤  ├─────────────┤
            └─ │Content Block│ ──► Optional since some entries only contain metadata
               └─────────────┘
                   .....
               ┌─────────────┐
               │End Block    │ ──► Required. Always be 2 512B blocks
               └─────────────┘
```

tar 文件的基础机构由 1 个或 N 个连续的 Tar Entry 组成，最后跟随 End Block 作为文件结束标志。每一个 Entry 为一个文件，一个文件夹，一个链接，又或者是一个设备。

tar 文件由于最早是一种以磁带介质作为存储的格式，当时部分磁带只支持以固定的大小（512B）进行写入，所以 tar 格式是由一个一个 512B 的块组成的，每一个块不足 512B 时会补 0 补到 512B 的倍数。

一个 Entry 包含两部分，Header Block 与 Content Block。Header Block 存储元信息，Content Block 存储文件内容。部分不含有正文内容的类型（如文件夹）不含有 Content Block。

一个 tar 文件在解压时，会从头到尾依次遍历每一个 Entry，并且尝试将其释放到对应的位置。

### Header Block

存储文件的元信息，包括文件名（含路径）、uid、gid、mode、mtime、content size 等属性。
作为一种颇有历史的文件存储格式，header 存储有一些有意思的特性：

- 固定 512B 长度；真正有意义的内容正好 500B，剩余 12B 为 padding。
- header 中每一个字段都是字符串，数字字段需要转换为 8 进制表示并且以字符串存储，如果使用二进制查看器可以很方便的看到每一个 field 的内容。
- 有的字段拥有奇怪的结束符。大部分以 \0（空 Byte，c 字符串规范）结束，但是有空格结束的，有空格 + \0 结束的，甚至还有 \0 + 空格结束的。不过现代 tar 工具对于结束符有一定的容错。

header 中存在对 header 本身的简单校验，通过将整个 header 每一个 Byte 作为整数相加，结果就是校验和（checksum）。该校验安全性低，并且只校验 header 不校验 content，用现代的目光来看比较过时。

由于每一个字段都有长度限制，因此对于部分长度敏感的字段存储量有限。如文件名（包含路径）最多存储 100B，和 `prefix` 一起存储最大也只能能到 255B 的长度，无法正常支持嵌套极深的路径。pax 扩展解决了该问题，下详。

### Content Block

文件内容。在 header 部分定义了一个 entry 的类型，真正有内容的类型（如普通文件）才会填充 Content Block。长度会 padding 到 512B 的倍数。

## pax

pax 格式扩展了 UStar 格式的限制，主要的优势为消除了原始 header 字段中的长度限制，并且允许给任意 entry 或者整个 tar 文件附加任何元信息字段。

pax 完全遵守 UStar 的格式规范，唯一的变化为在 header 中支持了两种新的扩展类型，`g` 与 `x`，通过在特定的位置增加一个 tar entry 来实现元信息的扩展。

- `g` entry：全文件最多只有一个，会放在整个文件的开头，用于定义整个 tar 文件都会遵守的属性，如字符集。
- `x` entry：全文件可以有多个，用于给下一个 entry 附加元信息。

pax entry 的内容为一个个的 k-v 对，通过特定的格式在 entry 的 Content Block 中排开。部分 key 用于覆盖 / 扩展 UStar 的某个 header，在解析对应被修改的 entry 时会优先使用 pax 提供的值。比如 pax 提供了一个 path key，如果存在，在解析时会覆盖对应 entry 的 prefix + name 的组合。

```
                  ┌─────────────┐
               ┌─ │Header Block │
Pax(g) Entry ◄─┤  ├─────────────┤ Modify the whole file
               └─ │Content Block│          │
                  └─────────────┘          ▼
                      .....
                  ┌─────────────┐
               ┌─ │Header Block │
Pax(x) Entry ◄─┤  ├─────────────┤ Modify the next entry
               └─ │Content Block│          │
                  ├─────────────┤          │
               ┌─ │Header Block │          │
   Tar Entry ◄─┤  ├─────────────┤ ◄────────┘
               └─ │Content Block│
                  └─────────────┘
                      .....
```
