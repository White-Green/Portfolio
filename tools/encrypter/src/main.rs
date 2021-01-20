use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write};

use clap::{App, Arg};
use sha3::Digest;

fn main() {
    let matches = App::new("encrypter")
        .arg(Arg::with_name("key")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("from")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("to")
            .takes_value(true)
            .required(true))
        .get_matches();
    let key = sha3::Sha3_256::digest(matches.value_of("key").unwrap().as_bytes()).to_vec().try_into().unwrap();
    let key = aes::Key::AES256(key);
    let mut from = File::open(matches.value_of("from").unwrap()).expect("unable to open from file");
    let mut to = File::open(matches.value_of("to").unwrap()).or_else(|_| File::create(matches.value_of("to").unwrap())).expect("unable to open to file");

    let mut vec = Vec::new();
    from.read_to_end(&mut vec).expect("unable to read from file");
    to.write_all(&aes::encrypt(&vec, &key)).expect("unable to write to file");
}
