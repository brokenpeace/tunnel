/**
 * File: src/main.rs
 * Author: Anicka Burova <anicka.burova@gmail.com>
 * Date: 04.09.2017
 * Last Modified Date: 04.09.2017
 * Last Modified By: Anicka Burova <anicka.burova@gmail.com>
 */
extern crate aws_sdk_rust;
extern crate ini;
extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::io::{self};

pub struct RawCodec;

use tokio_io::codec::{Encoder, Decoder};
use bytes::BytesMut;

impl Decoder for RawCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if buf.len() > 0 {
            let size = buf.len();
            let line = buf.split_to(size);
            Ok(Some(line.to_vec()))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for RawCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg);
        Ok(())
    }
}

use tokio_io::{AsyncRead};
use futures::{future, Future};

fn s3run() -> io::Result<()> {
    use ini::Ini;
    Ini::load_from_file("/home/anca/.s3cfg")
        .map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, err)
        })
        .and_then(|cfg| {
            // read s3 configuration from config file.
            cfg.section(Some("default".to_owned()))
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Cannot read default section"))
                .and_then(|section| {
                    section.get("access_key")
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Cannot read access_key value"))
                        .and_then(|access_key| {
                            section.get("secret_key")
                                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Cannot read secret_key value"))
                                .and_then(|secret_key| {
                                    Ok((access_key, secret_key))
                                })
                        })
                        .and_then(|(access_key, secret_key)| {
                            section.get("bucket_location")
                                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Cannot read bucket_location value"))
                                .and_then(|region| {
                                    Ok((access_key, secret_key, region))
                                })
                        })
                })
            .and_then(|(access_key, secret_key, region)| {
                // create connection to s3
                println!("{}\n{}\n{}", access_key, secret_key, region);
                use aws_sdk_rust::aws::common::credentials::{DefaultCredentialsProvider,ParametersProvider};
                ParametersProvider::with_parameters(
                    access_key.to_owned(),
                    secret_key.to_owned(),
                    None)
                    .and_then(|credentials| {
                        DefaultCredentialsProvider::new(Some(credentials))
                    })
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    .and_then(|provider| {
                        use std::str::FromStr;
                        use aws_sdk_rust::aws::common::region::Region;
                        Region::from_str(region)
                            .and_then(|region| {
                                use aws_sdk_rust::aws::s3::endpoint::{Endpoint, Signature};
                                Ok(Endpoint::new(region, Signature::V4, None, None, None, None))
                            })
                            .and_then(|endpoint| {
                                use aws_sdk_rust::aws::s3::s3client::S3Client;
                                Ok(S3Client::new(provider, endpoint))
                            })
                            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    })
            })
            .and_then(|client| {
                let bucket_name = "bucket_name";
                use aws_sdk_rust::aws::s3::object::PutObjectRequest;
                let mut object = PutObjectRequest::default();
                object.bucket = bucket_name.to_string();
                object.key = "exchange/tunnel.in".to_string();
                object.body = Some(b"this is a test.");
                match client.put_object(&object, None) {
                    Ok(output) => println!( "{:#?}", output),
                    Err(e) => println!("{:#?}", e),
                }
                // read s3 files
                use aws_sdk_rust::aws::s3::object::GetObjectRequest;
                let mut object = GetObjectRequest::default();
                object.bucket = bucket_name.to_string();
                object.key = "exchange/tunnel.out".to_string();
                use std::str;
                match client.get_object(&object, None) {
                    Ok(output) => println!( "\n\n{:#?}\n\n", str::from_utf8(&output.body).unwrap()),
                    Err(e) => println!( "{:#?}", e),
                }
                Ok(())
            })
        })
}

fn main() {
    //let _ = s3run().unwrap();
    let address = format!("0.0.0.0:{}", Some("1234").unwrap()).parse().unwrap();
    use tokio_core::reactor::Core;
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    use tokio_core::net::TcpListener;
    let listener = TcpListener::bind(&address, &handle).unwrap();
    use futures::stream::Stream;
    let server = listener
        .incoming()
        .for_each(move |(socket, _peer_addr)| {
            println!("server: connected");
            let (writer, reader) = socket.framed(RawCodec).split();
            //let address = "192.168.1.10:22".parse().unwrap();
            //
            let address = "10.10.101.146:22".parse().unwrap();
            use tokio_core::net::TcpStream;
            let client = TcpStream::connect(&address, &handle);
            let handle = handle.clone();
            client.and_then(move |socket| {
                let (writer2, reader2) = socket.framed(RawCodec).split();
                println!("client: connected");
                use futures::Sink;
                use std::str;
                let reader2 = reader2
                    .and_then(|data| {
                        match str::from_utf8(&data) {
                            Ok(s) => {
                                println!("client: {}", s);
                            }
                            Err(_) => {
                                println!("client: {}", data.len());
                            }
                        }
                        Box::new(future::ok(data))
                    });
                let reader = reader
                    .and_then(|data| {
                        match str::from_utf8(&data) {
                            Ok(s) => {
                                println!("server: {}", s);
                            }
                            Err(_) => {
                                println!("server: {}", data.len());
                            }
                        }
                        Box::new(future::ok(data))
                    });
                let server = writer.send_all(reader2).then(|_|Ok(()));
                let client = writer2.send_all(reader).then(|_|Ok(()));
                handle.spawn(server);
                handle.spawn(client);
                Ok(())
            })
        });
    let _ = core.run(server);
}
