use std::io::{Write, Result, Error};
use std::num::{NonZeroU8, NonZeroU32};

use libpd_rs::convenience::{PdGlobal, calculate_ticks};
use shout::ShoutConn;
use vorbis_rs::VorbisEncoderBuilder;

struct ShoutConnWriter(ShoutConn);

impl Write for ShoutConnWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.0.send(buf) {
            Ok(..) => {
                self.0.sync();
                return Ok(buf.len());
            },
            Err(..) => Err(Error::other("Error writing to Shoutcast Connection"))
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

fn main() {

    let sample_rate: NonZeroU32 = NonZeroU32::new(u32::try_from(44100).unwrap()).unwrap();
    let output_channels: NonZeroU8 = NonZeroU8::new(u8::try_from(2).unwrap()).unwrap();

    let mut pd = PdGlobal::init_and_configure(0, 2, 44100).unwrap();

    // Let's evaluate a pd patch.
    // We could have opened a `.pd` file also.
    // This patch would play a sine wave at 440hz.
    pd.eval_patch(
        r#"
    #N canvas 577 549 158 168 12;
    #X obj 23 116 dac~;
    #X obj 23 17 osc~ 440;
    #X obj 23 66 *~ 0.1;
    #X obj 81 67 *~ 0.1;
    #X connect 1 0 2 0;
    #X connect 1 0 3 0;
    #X connect 2 0 0 0;
    #X connect 3 0 0 1;
        "#,
    ).unwrap();

    let conn = shout::ShoutConnBuilder::new()
        .host(String::from("localhost"))
        .port(8000)
        .user(String::from("source"))
        .password(String::from("hackme"))
        .mount(String::from("/test.ogg"))
        .protocol(shout::ShoutProtocol::HTTP)
        .format(shout::ShoutFormat::Ogg)
        .build().unwrap();

    let conn_sink = ShoutConnWriter(conn);
    println!("Connected to server");

    let mut vencoder = VorbisEncoderBuilder::new(sample_rate, output_channels, conn_sink).unwrap().build().unwrap();

    // Turn audio processing on
    pd.activate_audio(true).unwrap();

    const BLOCK_SIZE: usize = 4096;
    let mut left_samps: [f32; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut right_samps: [f32; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut pd_output: [f32; BLOCK_SIZE * 2] = [0.0; BLOCK_SIZE * 2];
    loop {
        let ticks = calculate_ticks(2, pd_output.len() as i32);
        libpd_rs::process::process_float(ticks, &[], &mut pd_output);
        for i in 0..BLOCK_SIZE {
            left_samps[i] = pd_output[i * 2];
            right_samps[i] = pd_output[(i * 2) + 1];
        }
        let buffer: [&[f32; BLOCK_SIZE]; 2] = [&left_samps, &right_samps];
        vencoder.encode_audio_block(buffer).unwrap();
    }
    //println!("Finished!");


    //pd.activate_audio(false)?;

    //pd.close_patch()?;

}
