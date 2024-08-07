use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    Aes128Gcm, Key, Nonce,
};
use esp_idf_hal::{delay::FreeRtos, gpio, peripherals::Peripherals, prelude::*, uart};
use esp_idf_svc::log::*;
use labrador_ldpc::LDPCCode;
use std::time::{Duration, Instant};
use std::vec;
use typenum;

const PRIVATE_KEY: &str = "0123456789abcdef"; // 128 bits = 16 bytes

fn encrypt(private_key: &str, plain_text: &str) -> Vec<u8> {
    let key = Key::<Aes128Gcm>::from_slice(private_key.as_bytes());
    let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes128Gcm::new(key);

    let cipher_text: Vec<u8> = cipher
        .encrypt(&nonce, plain_text.as_bytes())
        .expect("failed to encrypt");

    let mut encrypted_data: Vec<u8> = nonce.to_vec();
    encrypted_data.extend_from_slice(&cipher_text);

    return encrypted_data;
}

fn decrypt(private_key: &str, cipher_text: Vec<u8>) -> String {
    let key = Key::<Aes128Gcm>::from_slice(private_key.as_bytes());

    let (nonce_arr, cipher_data) = cipher_text.split_at(12);

    let nonce: &GenericArray<u8, typenum::U12> = Nonce::from_slice(nonce_arr);

    let cipher = Aes128Gcm::new(key);

    let plaintext: Vec<u8> = cipher
        .decrypt(nonce, cipher_data)
        .expect("failed to decrypt");

    return String::from_utf8(plaintext).expect("failed to convert to string");
}

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();

    let rx: gpio::Gpio3 = peripherals.pins.gpio3;
    let tx: gpio::Gpio1 = peripherals.pins.gpio1;

    let uart_config: uart::config::Config =
        uart::config::Config::default().baudrate(Hertz(115_200));

    let uart_driver: uart::UartDriver = uart::UartDriver::new(
        peripherals.uart0,
        tx,
        rx,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyIOPin>::None,
        &uart_config,
    )
    .unwrap();

    let ldpc: LDPCCode = LDPCCode::TM2048;

    let plain_text: &str = "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"; // 100 bytes

    // 100 bytes of plain text data produces 128 bytes of cipher text

    let data: Vec<u8> = encrypt(PRIVATE_KEY, plain_text); // k = 1024 bits = 128 bytes

    // let data: Vec<u8> = (0..128).collect();  // k = 1024 bits = 128 bytes
    let mut codeword: Vec<u8> = vec![0u8; ldpc.n() / 8]; // n = 2048 bits = 256 bytes

    println!(" Actual data : {:?}", data);
    println!();

    let mut start: Instant = Instant::now();
    ldpc.copy_encode(&data, &mut codeword);
    let mut duration: Duration = start.elapsed();

    println!(" Encoding Time : {:?}", duration);
    println!();

    for j in data.iter() {
        print!("{:08b} ", j);
    }

    println!();
    println!();

    for i in codeword.iter() {
        print!("{:08b} ", i);
    }

    println!();
    println!();

    println!(" In Device 2");
    println!();

    let mut rx_codeword: Vec<u8> = codeword.clone();
    rx_codeword[1] ^= 0b00000000;

    for j in rx_codeword.iter() {
        print!("{:08b} ", j);
    }

    println!();
    println!();

    let mut working_space: Vec<u8> = vec![0u8; ldpc.decode_bf_working_len()];
    let mut rx_data: Vec<u8> = vec![0u8; ldpc.output_len()];

    start = Instant::now();
    ldpc.decode_bf(&rx_codeword, &mut rx_data, &mut working_space, 20);
    duration = start.elapsed();

    println!(" Decoding Time : {:?}", duration);
    println!();

    let actual_data: Vec<u8> = rx_data[..128].to_vec();

    println!(" Actual data : {:?}", actual_data);
    println!();

    for i in actual_data.iter() {
        print!("{:08b} ", i);
    }

    println!();

    assert_eq!(&data, &actual_data);

    let message: String = decrypt(PRIVATE_KEY, actual_data);

    println!(" Decrypted message : {}", message);

    assert_eq!(&message, &plain_text);

    // let mut cli_buf: Vec<u8> = Vec::new();

    loop {
        // let mut buf: Vec<u8> = vec![0u8; 128];
        // uart_driver.write(b"hee").unwrap();
        // match uart_driver.read(&mut buf, 1000) {
        //     Ok(bytes_read) => {
        //         if bytes_read > 1 {
        //             uart_driver.write(b"hello").unwrap();
        //         }
        //     }
        //     Err(_) => {}
        // }
        FreeRtos::delay_ms(50000);
        uart_driver.write(b"hello").unwrap();
    }
}
