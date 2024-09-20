import csv
import dns.resolver
import socks
import socket
import configparser
import os


# 配置 SOCKS5 代理
def configure_proxy(proxy_host, proxy_port):
    socks.set_default_proxy(socks.SOCKS5, proxy_host, int(proxy_port))
    socket.socket = socks.socksocket


# DNS 查询函数（不使用代理）
def reverse_lookup(domain):
    try:
        result = dns.resolver.resolve(domain, 'A')
        return [ip.address for ip in result]  # 返回所有的 IP 地址
    except dns.resolver.NXDOMAIN:
        return None


# DNS 查询函数（使用代理）
def reverse_lookup_with_proxy(domain):
    try:
        result = dns.resolver.resolve(domain, 'A')
        return [ip.address for ip in result]  # 返回所有的 IP 地址
    except dns.resolver.NXDOMAIN:
        return None


# 批量反查函数
def batch_reverse_lookup(input_file, output_file, proxy_host=None, proxy_port=None):
    # 启动提示信息
    if proxy_host and proxy_port:
        print("开始批量域名反查，并加载代理设置: {}:{}...".format(proxy_host, proxy_port))
    else:
        print("开始批量域名反查，未使用代理...")

    # 如果提供了代理配置，则启用代理
    if proxy_host and proxy_port:
        configure_proxy(proxy_host, proxy_port)
        lookup_function = reverse_lookup_with_proxy
    else:
        lookup_function = reverse_lookup

    with open(input_file, 'r', encoding='utf-8') as f:
        domains = f.readlines()

    domain_ip_map = {}

    for domain in domains:
        domain = domain.strip()
        all_ips = set()  # 使用 set 来去重
        for _ in range(3):  # 尝试多次查询
            ip_list = lookup_function(domain)  # 使用选择的查询函数
            if ip_list:
                all_ips.update(ip_list)
        if all_ips:
            domain_ip_map[domain] = all_ips
        else:
            domain_ip_map[domain] = {'No records found'}

    # 写入CSV文件，使用 gbk 编码
    with open(output_file, 'w', newline='', encoding='gbk') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['域名', 'IP 地址'])  # 写入标题行

        for domain, ips in domain_ip_map.items():
            writer.writerow([domain, ', '.join(ips)])  # 将域名和IP地址写入CSV

    # 完成提示信息
    print(f"批量域名反查完成，结果已保存至 {output_file}")


# 检查并生成默认的 config.ini 文件
def check_and_generate_config():
    config_file = 'config.ini'

    if not os.path.exists(config_file):
        config = configparser.ConfigParser()
        config['proxy'] = {
            'host': '',  # 默认代理IP
            'port': ''  # 默认代理端口
        }
        with open(config_file, 'w') as configfile:
            config.write(configfile)
        print(f'配置文件 {config_file} 已生成。')
    else:
        print(f'配置文件 {config_file} 已存在。')

    # 读取配置文件
    config = configparser.ConfigParser()
    config.read(config_file)

    # 返回配置文件中的代理设置
    host = config.get('proxy', 'host', fallback=None)
    port = config.get('proxy', 'port', fallback=None)

    # 返回的port需要转换成整数
    return host, port if port == '' else int(port)


if __name__ == '__main__':
    # 检查并生成 config.ini
    proxy_host, proxy_port = check_and_generate_config()

    # 启动时提示
    print("启动域名反查工具...")

    # 手动输入文件路径
    input_file = input("请输入要读取的域名文件路径（如 domains.txt）：")

    # 设置输出文件的默认路径
    output_file = 'results.csv'

    # 调用批量反查函数
    batch_reverse_lookup(input_file, output_file, proxy_host, proxy_port)
