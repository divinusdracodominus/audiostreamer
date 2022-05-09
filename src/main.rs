#![feature(vec_into_raw_parts)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, OutputCallbackInfo, SampleRate};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

/*
Current issue: not all data is being transmitted
Solution: have a thread shared buffer, one thread
receives from the network and writes to it
then playback thread grabs only the samples it needs
and writes zero of not enough available.
*/

#[derive(StructOpt)]
struct Args {}
// this will be a stupid simple implementation
fn main() {
    let mut buffer = Arc::new(Mutex::new(Vec::new()));
    let mut server = UdpSocket::bind("0.0.0.0:6432").unwrap();
    let mut client = UdpSocket::bind("0.0.0.0:3232").unwrap();
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    let input = host.default_input_device().unwrap();
    let mut data = Vec::with_capacity(96_000 * 100);
    unsafe {
        data.set_len(96_000 * 100);
    }
    let mut packet_num = 1;

    let output_config = output
        .supported_output_configs()
        .expect("failed to get output configs")
        .map(|c| c.with_sample_rate(SampleRate(48000)))
        .next()
        .unwrap();
    let input_config = input
        .supported_input_configs()
        .expect("failed to get input configs")
        .map(|c| c.with_sample_rate(SampleRate(48000)))
        .next()
        .unwrap();

    println!("output config: {:?}", output_config);
    println!("input config: {:?}", input_config);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6432);
    send_packet(&mut packet_num, addr, &client, &mut data);

    let in_stream = input
        .build_input_stream(
            &input_config.config(),
            move |data: &[i16], _| unsafe {
                let mut vec = data.to_vec();
                let (ptr, len, cap) = vec.into_raw_parts();
                let mut outvec: Vec<u8> = Vec::from_raw_parts(ptr as *mut u8, len * 2, cap * 2);
                send_packet(&mut packet_num, addr, &client, &mut outvec);
            },
            move |err| {
                panic!("{}", err);
            },
        )
        .unwrap();
    let cloned_buf = buffer.clone();
    let out_stream = input
        .build_output_stream(
            &output_config.config(),
            move |data: &mut [i16], _: &_| {
                let mut buf = cloned_buf.lock().unwrap();
                let indata = if buf.len() < data.len() {
                    let len = buf.len();
                    let mut newbuf = buf.drain(0..len).collect::<Vec<i16>>();
                    let mut idx = buf.len();
                    while idx < data.len() {
                        newbuf.push(0);
                        idx += 1;
                    }
                    newbuf
                }else{
                    buf.drain(0..data.len()).collect::<Vec<i16>>()
                };
                println!("indata len: {}, data len: {}", indata.len(), data.len());
                println!("indata: {:?}", indata);
                for (idx, val) in indata.iter().enumerate() {
                    data[idx] = *val / 100;
                }
            },
            move |err| {
                panic!("{}", err);
            },
        )
        .unwrap();

    /*let mut back = recv_data(&mut server);
    println!("data len: {:?}", back.len());
    back.extend_from_slice(&recv_data(&mut server));
    println!("data len: {:?}", back.len());*/
    in_stream.play();
    out_stream.play();
    
    loop {
        let mut readdata = recv_data(&mut server);
        println!("readdata length: {}", readdata.len());
        let (mut length, mut num) = decode(&mut readdata);
        println!("decoded length: {}", length);
        let mut indata: Vec<i16> = unsafe {
            let (ptr, len, cap) = readdata.into_raw_parts();
            Vec::from_raw_parts(ptr as *mut i16, len / 2, cap / 2)
        };
        let mut buf = buffer.lock().unwrap();
        buf.extend_from_slice(&indata);
    }
}

// returns (length, packet_number) draining
// them from the vector
fn decode(data: &mut Vec<u8>) -> (u16, u16) {
    let length: u16 = u16::from_le_bytes(data[0..2].try_into().unwrap());
    let packet_num: u16 = u16::from_le_bytes(data[2..4].try_into().unwrap());
    data.drain(0..4);
    data.drain((length as usize)..data.len());
    (length, packet_num)
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
}

fn send_packet(packet_num: &mut u16, addr: SocketAddr, socket: &UdpSocket, data: &mut Vec<u8>) {
    println!("sending packet of length: {}", data.len());
    let mut index = 0;
    let len = data.len();
    while (index < len) {
        let pack_num = packet_num.to_be_bytes();
        let end = if 48_000 > data.len() {
            data.len()
        } else {
            48_000
        };
        let l = end as u16;

        let mut send_data = data.drain(0..end).collect::<Vec<u8>>();
        send_data.extend_from_slice(&pack_num);
        send_data.extend_from_slice(&l.to_be_bytes());
        // so the header is at the front of the vector
        send_data.reverse();
        index += end;
        socket.send_to(&send_data.as_slice(), addr).unwrap();

        *packet_num = (*packet_num) + 1;
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
