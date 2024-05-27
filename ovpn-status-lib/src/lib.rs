use std::fs::File;
use std::io;
use std::io::BufRead;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Client {
    pub common_name: String,
    real_address: SocketAddr,
    pub virtual_address: IpAddr,
    virtual_address_ipv6: Option<IpAddr>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    connected_since: NaiveDateTime,
    pub connected_since_time_t: u64,
    username: String,
    client_id: u32,
    peer_id: u32,
    data_channel_cipher: String
}

#[derive(Debug)]
pub struct Route {
    virtual_address: IpAddr,
    common_name: String,
    real_address: SocketAddr,
    last_ref: NaiveDateTime,
    last_ref_time_t: u64
}


pub struct Title {
    version: String,
    arch: String,
    options: Vec<String>,
}

pub struct Status {
    pub snapshot_time: Option<SnapshotTime>,
    pub client_list: Vec<Client>,
    routing_table: Vec<Route>,
    title: Option<Title>
}

pub struct SnapshotTime {
    snapshot_datetime: NaiveDateTime,
    pub snapshot_time_t: u64
}

pub fn process_title(title_parts: Vec<String>) -> Title{
    let flags = title_parts[2..].to_vec();
    Title{
        version: title_parts[1].clone(),
        arch: title_parts[2].clone(),
        options: flags
    }
}


pub fn process_client(client_parts: Vec<String>) -> Client {

    let real_address_socket: SocketAddrV4 = client_parts[1].parse().unwrap();
    let virtual_address_socket: Ipv4Addr = client_parts[2].parse().unwrap();

    Client{
        common_name: client_parts[0].clone(),
        real_address: SocketAddr::from(real_address_socket),
        virtual_address: IpAddr::from(virtual_address_socket),
        virtual_address_ipv6: {
            match client_parts[3].parse::<IpAddr>() {
               Ok(address_string) => Some(IpAddr::from(address_string)),
               Err(_error) => None
            }
        },
        bytes_received: client_parts[4].parse().unwrap(),
        bytes_sent: client_parts[5].parse().unwrap(),
        connected_since: NaiveDateTime::parse_from_str(client_parts[6].as_str(), "%Y-%m-%d %H:%M:%S").unwrap(),
        connected_since_time_t: client_parts[7].parse().unwrap(),
        username: client_parts[8].clone(),
        client_id: client_parts[9].parse().unwrap(),
        peer_id: client_parts[10].parse().unwrap(),
        data_channel_cipher: client_parts[11].clone(),
    }
}

pub fn process_route(route_paths: Vec<String>) -> Route {
    let real_address: SocketAddrV4 = route_paths[2].parse().unwrap();
    let virtual_address: Ipv4Addr = route_paths[0].parse().unwrap();

    Route{
        virtual_address: IpAddr::from(virtual_address.clone()),
        common_name: route_paths[1].clone(),
        real_address: SocketAddr::from(real_address.clone()),
        last_ref: NaiveDateTime::parse_from_str(route_paths[3].as_str(), "%Y-%m-%d %H:%M:%S").unwrap(),
        last_ref_time_t: route_paths[4].parse().unwrap(),
    }

}

pub fn process_status_file(status_file_path: &str)  -> Status {
    let mut status = Status{
        snapshot_time: None,
        client_list: Vec::new(),
        routing_table: Vec::new(),
        title: None,
    };
    if let Ok(lines) = read_lines(status_file_path) {
        for line in lines.flatten() {
            let res: Vec<String> = line.split(",").map(|s| s.to_string()).collect();
            match  res[0].as_str() {
                "TITLE" => {
                    let parts: Vec<String> = res.get(1).unwrap().to_string().split(" ").map(|s| s.to_string()).collect();
                    status.title = Option::from(process_title(parts));
                }
                "TIME" => {
                    status.snapshot_time = Some(SnapshotTime {
                        snapshot_datetime: NaiveDateTime::parse_from_str(res[1].as_str(), "%Y-%m-%d %H:%M:%S").unwrap(),
                        snapshot_time_t: res[2].parse().unwrap(),
                    })
                }
                "HEADER" => {}
                "CLIENT_LIST" => {
                    let parts: Vec<String> = res.split_at(1).1.to_vec();

                    status.client_list.push(process_client(parts));
                }
                "ROUTING_TABLE" => {
                    let parts: Vec<String> = res.split_at(1).1.to_vec();
                    status.routing_table.push(process_route(parts));
                }
                "GLOBAL_STATS" => {}
                "END" => {}
                _ => {}
            }
        }
    }
    status
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_process_title(){
        let line = "OpenVPN 2.6.3 aarch64-unknown-linux-gnu [SSL (OpenSSL)] [LZO] [LZ4] [EPOLL] [PKCS11] [MH/PKTINFO] [AEAD] [DCO]";
        let parts: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
        let title = process_title(parts);
        assert_eq!(title.arch, "aarch64-unknown-linux-gnu".to_string());
        assert_eq!(title.version, "2.6.3".to_string());
    }

    #[test]
    fn test_proces_client(){
        let line = "daubers,85.119.83.103:33888,10.8.0.10,,410675,272926,2024-05-05 19:36:50,1714934210,UNDEF,0,0,AES-256-GCM";
        let parts: Vec<String> = line.split(",").map(|s| s.to_string()).collect();
        let client = process_client(parts);
        assert_eq!(client.common_name, "daubers".to_string());
        assert_eq!(client.real_address, SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(85,119,83,103),33888)));
        assert_eq!(client.connected_since_time_t, 1714934210);
        assert_eq!(client.virtual_address, IpAddr::from(Ipv4Addr::new(10,8,0,10)))

    }

    #[test]
    fn test_all(){
        process_status_file("test_data/status_sample_1.log");

    }
}
