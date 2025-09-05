# Domain_Seek
**批量域名反查工具（HTTP DNS API + 代理 + CNAME递归）**

### Options

| 参数 | 说明 |
|------|------|
| `-i, --input <INPUT>` | 输入 CSV 文件路径 (第一列为域名)<br>[default: domains.csv] |
| `-o, --output <OUTPUT>` | 输出 XLSX 文件路径<br>[default: result.xlsx] |
| `-p, --proxy <PROXY>` | 代理服务器 (http / socks5) |
| `-c, --concurrency <CONCURRENCY>` | 并发查询数量，默认 50<br>[default: 50] |
| `-d, --max-depth <MAX_DEPTH>` | CNAME递归最大深度，默认 5<br>[default: 5] |
| `-h, --help` | Print help |
| `-V, --version` | Print version |


