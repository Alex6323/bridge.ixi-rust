mod constants;

include!(concat!(env!("OUT_DIR"), "/bridge.rs"));

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut, IntoBuf};
use prost::*;

use std::io::Cursor;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use crate::constants::*;
use wrapper_message::{MessageType, Msg};

fn main() {
    // Establish connection to Bridge.ixi
    let addr: SocketAddr = format!("{}:{}", BRIDGE_IP, BRIDGE_PORT).parse().unwrap();
    let mut stream = TcpStream::connect(&addr).expect("Couldn't connect to bridge");
    println!("Successfully connected to Bridge.ixi");

    // Create a transaction
    let mut tx_builder = TransactionBuilder::default();
    tx_builder.address =
        "TEST9ADDRESS999999999999999999999999999999999999999999999999999999999999999999999".into();
    tx_builder.tag = "BRIDGE9TEST".into();

    // Create a submit-transaction request
    let submit_req = SubmitTransactionBuilderRequest {
        transaction_builder: Some(tx_builder),
    };

    // Wrap the request
    let mut msg = WrapperMessage::default();
    msg.set_message_type(MessageType::SubmitTransactionBuilderRequest);
    msg.msg = Some(Msg::SubmitTransactionBuilderRequest(submit_req));

    send_message(&mut stream, msg);
    println!("Sucessfully sent submit-transacton request");

    thread::sleep(Duration::from_millis(1000));

    // Create a find-transaction request
    let find_req = FindTransactionsByAddressRequest {
        address:
            "TEST9ADDRESS999999999999999999999999999999999999999999999999999999999999999999999"
                .into(),
    };

    // Wrap the request
    let mut find_msg = WrapperMessage::default();
    find_msg.set_message_type(MessageType::FindTransactionsByAddressRequest);
    find_msg.msg = Some(Msg::FindTransactionsByAddressRequest(find_req));

    send_message(&mut stream, find_msg);
    println!("Sucessfully sent find-transacton request");

    thread::sleep(Duration::from_millis(1000));

    //
    let resp_msg = read_message(&mut stream);
    println!("Sucessfully received response: {}", resp_msg.message_type);

    if let Some(msg) = resp_msg.msg {
        if let Msg::FindTransactionsByAddressResponse(response) = msg {
            //println!("{:?}", response);
            for tx in response.transaction.iter() {
                println!("{} {}", tx.address, tx.tag);
            }
        }
    }
}

fn send_message(stream: &mut TcpStream, msg: WrapperMessage) {
    //use std::ops::Deref;
    let mut buf1 = BytesMut::with_capacity(2048);
    // Send the message
    msg.encode(&mut buf1).expect("error encoding message");
    let buf_len = buf1.len();
    //println!("{}", buf_len);
    //println!("{:?}", buf1.deref());

    let mut buf2 = BytesMut::with_capacity(2048);
    buf2.put_i32_be(buf_len as i32);
    buf2.put(buf1);

    stream
        .write_all(&buf2)
        .expect("error writing buf to stream");

    stream.flush().expect("error flushing bytes to stream");
}

fn read_message(stream: &mut TcpStream) -> WrapperMessage {
    let mut buf_len = [0u8; 4];
    stream
        .read_exact(&mut buf_len)
        .expect("error reading from stream");
    let mut rdr = Cursor::new(buf_len);
    let msg_len = rdr.read_i32::<BigEndian>().unwrap();
    //println!("{}", msg_len);

    //let mut buf = BytesMut::with_capacity(2875);
    let mut buf = vec![0u8; msg_len as usize];
    stream
        .read_exact(&mut buf)
        .expect("error reading from stream");

    let mut buf2 = BytesMut::with_capacity(msg_len as usize);
    buf2.put(&buf[..]);
    //println!("{:?}", buf2);

    //WrapperMessage::decode(buf2).expect("error decoding message")
    let mut resp_msg = WrapperMessage::default();
    resp_msg.merge(buf).expect("error reading response");
    resp_msg
}
