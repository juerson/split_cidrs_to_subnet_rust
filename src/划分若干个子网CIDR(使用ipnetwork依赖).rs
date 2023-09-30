use ipnetwork::IpNetwork;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::process;
use std::io::{self, BufRead, Write};
use std::collections::HashSet;
use std::cmp::Ordering;

fn main() -> Result<(), Box<dyn Error>> {
	println!("本工具：使用ipnetwork依赖库，批量将CIDR分割为若干个24位掩码的子网CIDR！");
    let file_cidrs = read_file("ips-v4.txt")?;
    let subnet_mask = "/24"; // 子网掩码

    let all_subnets: Vec<String> = file_cidrs
        .iter()
        .flat_map(|cidr_str| {
            if let Ok(ip_network) = cidr_str.parse::<IpNetwork>() {
                let ip_addresses: Vec<_> = ip_network.iter().collect();
                let chunked_ips: Vec<_> = ip_addresses.chunks(256).collect();
                chunked_ips
                    .iter()
                    .map(|chunk| {
                        let start_ip = chunk[0];
                        format!("{}{}", start_ip, subnet_mask)
                    })
                    .collect::<Vec<String>>()
            } else {
                Vec::new()
            }
        })
        .collect();
	let mut subnets: Vec<String> = all_subnets.into_iter().collect::<HashSet<_>>().into_iter().collect();
	
	// 排序subnets使用自定义比较函数
    subnets.sort_by(|a, b| compare_ipv4_cidr(a, b));
	
	
    // 打印所有子网
    for (index, subnet) in subnets.iter().enumerate() {
        println!("Subnet {}: {}", index + 1, subnet);
    }

    write_to_file("output_cidrs.txt", &subnets)?;

    wait_for_enter();
    Ok(())
}

fn read_file(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_e) => {
			println!("{}文件不存在，打开文件失败！", filename);
			wait_for_enter();
            process::exit(0);
		}
    };
    let cidrs: Vec<String> = io::BufReader::new(file)
        .lines()
        .filter_map(|line| {
            if let Ok(line) = line {
                let trimmed = line.trim().to_string();
                if is_valid_cidr(&trimmed) {
                    Some(trimmed)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if cidrs.is_empty() {
		println!("{}文件的内容为空，或者无CIDR!",filename);
		wait_for_enter();
		process::exit(0);
    }
    Ok(cidrs)
}

fn is_valid_cidr(input: &str) -> bool {
    let cidr_regex = Regex::new(r"^(?:\d{1,3}\.){3}\d{1,3}/\d{1,2}$").unwrap();
    cidr_regex.is_match(input)
}

fn write_to_file(filename: &str, cidrs: &[String]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;

    for cidr in cidrs {
        writeln!(file, "{}", cidr)?;
    }

    Ok(())
}

fn wait_for_enter() {
    print!("\nPress Enter to exit...");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
}


// 自定义比较函数，用于比较IPv4 CIDR地址，升序排序
fn compare_ipv4_cidr(cidr1: &str, cidr2: &str) -> Ordering {
    // 解析CIDR并提取IP地址的整数值
    fn parse_ip_address(cidr: &str) -> u32 {
        let parts: Vec<&str> = cidr.split('/').collect();
        let ip_parts: Vec<&str> = parts[0].split('.').collect();
        let ip: u32 = ip_parts.iter()
            .map(|&part| part.parse::<u32>().unwrap())
            .fold(0, |acc, val| (acc << 8) | val);
        ip
    }

    let ip1 = parse_ip_address(cidr1);
    let ip2 = parse_ip_address(cidr2);

    // 使用整数值进行比较
    ip1.cmp(&ip2)
}