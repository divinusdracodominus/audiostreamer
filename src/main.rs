use std::convert::TryInto;
use std::net::{SocketAddr, UdpSocket, IpAddr, Ipv4Addr};
use structopt::StructOpt;
use cpal::traits::HostTrait;

#[derive(StructOpt)]
struct Args {}
// this will be a stupid simple implementation
fn main() {
    let mut server = UdpSocket::bind("0.0.0.0:6432").unwrap();
    let mut client = UdpSocket::bind("0.0.0.0:3232").unwrap();
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    let input = host.default_input_device().unwrap();
    let mut data = Vec::with_capacity(48_000);
    unsafe { data.set_len( 48_000); }
    let mut packet_num = 1;
    
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6432);
    send_packet(&mut packet_num, addr, client, &mut data);
    let back = recv_data(&mut server);
    println!("data len: {:?}", back.len());
}

pub struct Header {
    data_len: u16,
    packet_num: u16,
}

impl Header {
    pub fn to_raw(&self) -> Vec<u8> {
        let mut outvec: Vec<u8> = Vec::with_capacity(4);
        outvec.extend_from_slice(&self.data_len.to_be_bytes());
        outvec.extend_from_slice(&self.packet_num.to_be_bytes());
        outvec
    }
    /*pub fn from_raw(data: &[u8]) -> Self {
        let data_len: u16 = data[0..2].try_into::<[u8;2]>().unwrap().from_be_bytes();
        let packet_num: u16 = data[2..4].try_into().unwrap().from_be_bytes();
        Self{
            data_len,
            packet_num,
        }
    }*/
}

fn send_packet(packet_num: &mut u16, addr: SocketAddr, socket: UdpSocket, data: &mut Vec<u8>) {
    let mut index = 0;
    let len = data.len();
    while (index < len) {
        let pack_num = packet_num.to_be_bytes();
        let end = if index + 48_000 > len {
            len
        } else {
            index + 48_000
        };
        let l = (end - index) as u16;

        let mut send_data = data.drain(index..index + end).collect::<Vec<u8>>();
        send_data.extend_from_slice(&pack_num);
        send_data.extend_from_slice(&l.to_be_bytes());
        // so the header is at the front of the vector
        send_data.reverse();
        index = end;
        socket.send_to(&send_data.as_slice(), addr).unwrap();
        *packet_num = *packet_num + 1u16;
    }
}

fn recv_data(socket: &mut UdpSocket) -> Vec<u8> {
    let mut buf: [u8; 48004] = [0; 48004];
    let (mut size, mut target) = socket.recv_from(&mut buf).unwrap();
    /*if (target == *addr && size <= 48004) {
        return buf.to_vec();
    }
    if (target == *addr && size > 48004) {
        panic!("packet fragmentation");
    }
    while (*addr != target) {
        (size, target) = socket.recv_from(&mut buf).unwrap();
        if size != 48004 {
            panic!("packet fragmentation");
        } 
    }*/
    // I really should grab size data
    buf.to_vec()
}
