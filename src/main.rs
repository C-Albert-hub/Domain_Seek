use std::{collections::{HashSet, HashMap}, error::Error, fs::File, path::PathBuf, sync::Arc};
use csv::ReaderBuilder;
use rust_xlsxwriter::Workbook;
use colored::*;
use clap::Parser;
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::{Client, Proxy};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about = "批量域名反查工具（HTTP DNS API + 代理）")]
struct Args {
    #[arg(short, long, default_value = "domains.csv", help = "输入 CSV 文件路径 (第一列为域名)")]
    input: PathBuf,
    #[arg(short, long, default_value = "result.xlsx", help = "输出 XLSX 文件路径")]
    output: PathBuf,
    #[arg(short, long, help = "代理服务器 (格式: http://127.0.0.1:1080 或 socks5://127.0.0.1:1080)")]
    proxy: Option<String>,
    #[arg(short='c', long="concurrency", default_value_t = 50, help="并发查询数量，默认 50")]
    concurrency: usize,
}

#[derive(Deserialize)]
struct GoogleDnsAnswer {
    data: Option<String>,
}

#[derive(Deserialize)]
struct GoogleDnsResponse {
    #[serde(rename = "Answer")]
    answer: Option<Vec<GoogleDnsAnswer>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("{}", "批量域名反查工具启动...".green().bold());
    println!("输入文件: {}", args.input.to_string_lossy().yellow());
    println!("输出文件: {}", args.output.to_string_lossy().cyan());
    println!("并发查询数量: {}", args.concurrency.to_string().magenta());
    if let Some(proxy) = &args.proxy {
        println!("使用代理: {}", proxy.magenta());
    }

    // 读取 CSV 并去重
    let file = File::open(&args.input)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);
    let mut domains: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for result in rdr.records() {
        let record = result?;
        if let Some(domain) = record.get(0) {
            let d = domain.trim().to_string();
            if seen.insert(d.clone()) {
                domains.push(d);
            }
        }
    }
    println!(
        "读取到域名数量: {} (去重后: {})",
        domains.len().to_string().yellow(),
        domains.len().to_string().cyan()
    );

    // 设置 HTTP 客户端
    let client_builder = Client::builder();
    let client = if let Some(proxy_str) = &args.proxy {
        let proxy = Proxy::all(proxy_str)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };
    let client = Arc::new(client);

    // 初始化 Excel
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "域名")?;
    worksheet.write_string(0, 1, "IP地址")?;

    // 并发控制
    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let mut tasks = FuturesUnordered::new();

    for domain in domains.iter() {
        let d = domain.clone();
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        tasks.push(tokio::spawn(async move {
            let url = format!("https://dns.google/resolve?name={}&type=A", d);
            let res = match client.get(&url).send().await {
                Ok(resp) => match resp.json::<GoogleDnsResponse>().await {
                    Ok(json) => json.answer.and_then(|ans| ans.get(0).and_then(|a| a.data.clone())),
                    Err(_) => None,
                },
                Err(_) => None,
            };
            drop(permit);
            (d, res)
        }));
    }

    // 收集结果
    let mut results_map: HashMap<String, Option<String>> = HashMap::new();
    while let Some(Ok((domain, ip))) = tasks.next().await {
        results_map.insert(domain, ip);
    }

    // 按原始顺序输出和写 Excel
    let mut row = 1;
    for domain in domains {
        let ip_opt = results_map.get(&domain).cloned().unwrap_or(None);
        match &ip_opt {
            Some(ip_str) => println!("{} -> {}", domain.blue(), ip_str.green()),
            None => println!("{} -> {}", domain.blue(), "查询失败".red()),
        }
        worksheet.write_string(row, 0, domain)?;
        worksheet.write_string(row, 1, &ip_opt.unwrap_or("查询失败".to_string()))?;
        row += 1;
    }

    workbook.save(&args.output)?;
    println!("结果已保存到 {}", args.output.to_string_lossy().bold().green());

    Ok(())
}
