use labrador_ldpc::LDPCCode;
use std::time::Instant;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let code: LDPCCode = LDPCCode::TM1280;

    let txdata: Vec<u8> = (0..128).collect();
    let mut txcode: Vec<u8> = vec![0u8; code.n() / 8];

    let mut i: u8 = 0;

    loop {
        if i > 6 {
            break;
        } else {
            i += 1;

            let mut start = Instant::now();
            code.copy_encode(&txdata, &mut txcode);
            let mut elapsed = start.elapsed();

            println!("encode time: {:?}", &elapsed);

            // println!("tx data: {:?}", &txdata);
            // println!("tx codeword: {:?}", &txcode);

            let mut rxcode = txcode.clone();
            rxcode[0] ^= 0x55;

            let mut working = vec![0u8; code.decode_bf_working_len()];
            let mut rxdata = vec![0u8; code.output_len()];

            start = Instant::now();
            code.decode_bf(&rxcode, &mut rxdata, &mut working, 20);
            elapsed = start.elapsed();

            println!("decode time: {:?}", &elapsed);

            // println!("rx codeword {:?}", &rxcode);
            // println!("rx data {:?}", &rxdata);

            assert_eq!(&rxdata[..128], &txdata[..128]);
        }
    }
}
