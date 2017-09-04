/**
 * File: src/main.rs
 * Author: Anicka Burova <anicka.burova@gmail.com>
 * Date: 04.09.2017
 * Last Modified Date: 04.09.2017
 * Last Modified By: Anicka Burova <anicka.burova@gmail.com>
 */
extern crate aws_sdk_rust;
extern crate ini;

use std::io::{self};


fn main() {
    use ini::Ini;
    let _ = Ini::load_from_file("/home/milan/.s3cfg")
        .map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, err)
        })
        .and_then(|cfg| {
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
                            .and_then(|_region| {
                                use aws_sdk_rust::aws::s3::endpoint::{Endpoint, Signature};
                                //Ok(Endpoint::new(region, Signature::V4, None, None, None, None))
                                Ok(Endpoint::new(Region::UsEast1, Signature::V4, None, None, None, None))
                            })
                            .and_then(|endpoint| {
                                use aws_sdk_rust::aws::s3::s3client::S3Client;
                                Ok(S3Client::new(provider, endpoint))
                            })
                            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    })
            })
            .and_then(|client| {
                client.list_buckets()
                    .and_then(|buckets| {
                        println!("{:#?}", buckets);
                        Ok(())
                    })
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            })
        })
    //let _ = load_credentials()
        //.and_then(|(credentials, region)| {
            //Ok(())
            ////region
                ////.parse()
                ////.map_err(|_| {
                    ////io::Error::new(io::ErrorKind::InvalidData, "Failed to parse region")
                ////})
                ////.and_then(|region| {
                    ////Ok(Bucket::new("imsdistributionfiles", region, credentials))
                ////})
        //})
        //.and_then(|bucket| {
            //Ok(())
            //bucket
                //.list("/exchange", None)
                //.and_then(|(list, code)| {
                    //println!("code = {}\nlist = {:?}", code, list);
                    //Ok(())
                //})
                //.map_err(|err| {
                    //println!("failed: {}", err.description());
                    //io::Error::new(io::ErrorKind::InvalidData, err.description().to_owned())
                //})
        //})
        .unwrap();
}
