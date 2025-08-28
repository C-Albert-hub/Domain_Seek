use std::{collections::HashSet, error::Error, fs::File, path::PathBuf};
use csv::ReaderBuilder;
use rust_xlsxwriter::Workbook;
use colored::*;
use tokio::net::lookup_host;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "批量域名反查工具", long_about = None)]
struct Args {
    /// 输入 CSV 文件路径 (第一列为域名)
    #[arg(short, long, default_value = "domains.csv", help = "指定输入 CSV 文件，例如 domains.csv")]
    input: PathBuf,

    /// 输出 XLSX 文件路径
    #[arg(short, long, default_value = "result.xlsx", help = "指定输出 XLSX 文件，例如 result.xlsx")]
    output: PathBuf,

    /// 代理服务器 (格式: ip:port)，例如 127.0.0.1:1080
    #[arg(short, long, help = "指定代理 IP 和端口，例如 127.0.0.1:1080")]
    proxy: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("{}", "批量域名反查工具启动...".green().bold());
    println!("输入文件: {}", args.input.to_string_lossy().yellow());
    println!("输出文件: {}", args.output.to_string_lossy().cyan());
    if let Some(proxy) = &args.proxy {
        println!("使用代理: {}", proxy.magenta());
    }

    // 读取 CSV
    let file = File::open(&args.input)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);

    let mut domains: HashSet<String> = HashSet::new();
    for result in rdr.records() {
        let record = result?;
        if let Some(domain) = record.get(0) {
            domains.insert(domain.trim().to_string());
        }
    }
    println!(
        "读取到域名数量: {} (去重后: {})",
        domains.len().to_string().yellow(),
        domains.len().to_string().cyan()
    );

    // 写 Excel
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, "域名")?;
    worksheet.write_string(0, 1, "IP地址")?;

    let mut row = 1;
    for domain in domains {
        print!("查询 {} ... ", domain.blue());

        // 系统 DNS 解析（暂时未走代理）
        match lookup_host((domain.as_str(), 0)).await {
            Ok(mut addrs) => {
                if let Some(ip) = addrs.next() {
                    println!("{}", "成功".green());
                    worksheet.write_string(row, 0, &domain)?;
                    worksheet.write_string(row, 1, &ip.ip().to_string())?;
                }
            }
            Err(e) => {
                println!("{} {}", "失败".red(), e);
                worksheet.write_string(row, 0, &domain)?;
                worksheet.write_string(row, 1, "查询失败")?;
            }
        }
        row += 1;
    }

    workbook.save(&args.output)?;
    println!("结果已保存到 {}", args.output.to_string_lossy().bold().green());

    Ok(())
}
