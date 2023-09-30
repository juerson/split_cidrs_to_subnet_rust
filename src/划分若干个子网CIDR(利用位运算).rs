use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process;
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr};
use regex::Regex;
use std::collections::HashSet;
use std::cmp::Ordering;


fn main() -> Result<(), Box<dyn Error>> {
	println!("本工具：利用位运算，批量将CIDR分割为若干个24位掩码的子网CIDR！");
    // 从文件中读取CIDR列表
    let filename = "ips-v4.txt";
    let cidrs = read_file(filename)?;

    // 打开输出文件
    let output_filename = "output_cidrs.txt";
    let mut output_file = File::create(output_filename)?;
	// 创建HashSet来存储子网CIDRs
    let mut all_subnets: HashSet<String> = HashSet::new(); 
	
    // 遍历CIDR列表并拆分子网，将每个子网写入输出文件
    for base_cidr in &cidrs {
		let subnets = split_subnets(base_cidr, 24); // 这里的24为要生成的CIDR后缀（/24）
		// 使用extend方法合并到all_subnets中
		all_subnets.extend(subnets);
    }
	// 将HashSet转换为Vec并排序
    let mut subnets_vec: Vec<String> = all_subnets.into_iter().collect();
	// 排序subnets使用自定义比较函数
    subnets_vec.sort_by(|a, b| compare_ipv4_cidr(a, b));
	
	// 遍历存储子网CIDR的向量，将其写入输出文件
    for (index, subnet) in subnets_vec.iter().enumerate() {
		println!("Subnet {}: {}",index + 1,subnet);
        writeln!(output_file, "{}", subnet)?;
    }
	println!("\n结果已经写入{}文件中！",filename);
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
	let unique_cidrs: Vec<String> = cidrs.into_iter().collect::<HashSet<_>>().into_iter().collect();

    if unique_cidrs.is_empty() {
		println!("{}文件的内容为空，或者没有有效的CIDR!",filename);
		wait_for_enter();
		process::exit(0);
    }
    Ok(unique_cidrs)
}

fn is_valid_cidr(input: &str) -> bool {
    let cidr_regex = Regex::new(r"^(?:\d{1,3}\.){3}\d{1,3}/\d{1,2}$").unwrap();
    cidr_regex.is_match(input)
}

// 分割一个CIDR为无数小块的CIDR，其中的suffix为要生成的后缀（/24）
fn split_subnets(base_cidr: &str, suffix: u8) -> Vec<String> {
    let ip = base_cidr.split('/').next().unwrap();
    let mask = base_cidr.split('/').last().unwrap().parse::<u8>().unwrap();

    let ip_addr = IpAddr::V4(Ipv4Addr::from_str(ip).unwrap());

    let ip_integer = match ip_addr {
        IpAddr::V4(v4) => u32::from_be_bytes(v4.octets()),
        IpAddr::V6(_) => unimplemented!("IPv6 not supported yet"),
    };

    let subnet_mask = !((1u32 << (32 - suffix)) - 1);

    let max_subnets = 1u32 << (suffix - mask); // 根据前缀计算最大子网数量

    let mut subnets = Vec::new();
    let mut subnet_ip = ip_integer & subnet_mask;

    for _ in 0..max_subnets {
        let subnet = Ipv4Addr::from(subnet_ip.to_be_bytes());
        subnets.push(format!("{}/{}", subnet, suffix));
        subnet_ip += 1u32 << (32 - suffix);
    }

    subnets
}

fn wait_for_enter() {
    print!("按Enter键退出程序...");
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