use std::collections::HashMap;
use std::str::FromStr;

use electrum_client::Client as ElectrumClient;

use tokio::runtime::Runtime;
use tokio::task;

use log::info;

use libp2ep::bitcoin::hashes::hex::FromHex;
use libp2ep::bitcoin::secp256k1::{All, Secp256k1};
use libp2ep::bitcoin::*;

use libp2ep::client::*;
use libp2ep::demo::*;

fn main() {
    env_logger::init();

    let mut rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();

    local.block_on(&mut rt, run());
}

async fn run() {
    let send_to = Address::from_str("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080").unwrap();
    let send_to_amount = 3_000_000;

    let secp: Secp256k1<All> = Secp256k1::gen_new();
    let sk = PrivateKey::from_str("cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy").unwrap();
    let address = Address::p2wpkh(&sk.public_key(&secp), Network::Regtest);
    info!("address: {}", address.to_string());

    let previous_output_value = 100_000_000;
    let previous_output = OutPoint {
        txid: Txid::from_hex("c790622f0b33ff5b99ee10f8cb4bfb9271390ed7cfeb596209be75fb6d86e088")
            .unwrap(),
        vout: 0,
    };
    let vin = TxIn {
        previous_output,
        sequence: 0xFFFFFFFF,
        ..Default::default()
    };
    let tx = Transaction {
        version: 2,
        lock_time: 0,
        input: vec![vin],
        output: vec![
            TxOut {
                script_pubkey: address.script_pubkey(),
                value: previous_output_value - send_to_amount - 5000,
            },
            TxOut {
                script_pubkey: send_to.script_pubkey(),
                value: send_to_amount,
            },
        ],
    };

    let mut meta_map = HashMap::new();
    meta_map.insert(
        tx.input[0].previous_output.clone(),
        (previous_output_value, address.script_pubkey()),
    );

    let electrum_client = ElectrumClient::new("kirsche.emzy.de:50001").unwrap();
    let electrum = ElectrumBlockchain::new(electrum_client);
    let signer = SoftwareSigner::new(sk, meta_map);

    let mut client = Client::new(
        "vzxfzi6dn6t64hvscbjn735joug4ln4wc4zfaxvifr26fqbgudflz6yd.onion:9000",
        electrum,
        signer,
        tx,
        1,
    )
    .await
    .unwrap();
    let txid = client.start().await.unwrap();

    info!("Completed with txid: {}", txid);
}
