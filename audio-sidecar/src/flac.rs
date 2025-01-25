use flac_sys::{
    FLAC__StreamEncoder, FLAC__StreamEncoderInitStatus_FLAC__STREAM_ENCODER_INIT_STATUS_OK,
    FLAC__StreamEncoderState, FLAC__stream_encoder_delete, FLAC__stream_encoder_finish,
    FLAC__stream_encoder_get_state, FLAC__stream_encoder_init_file, FLAC__stream_encoder_new,
    FLAC__stream_encoder_process, FLAC__stream_encoder_set_bits_per_sample,
    FLAC__stream_encoder_set_channels, FLAC__stream_encoder_set_compression_level,
    FLAC__stream_encoder_set_sample_rate,
};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::ptr;

pub struct EncoderConfig {
    output_path: Option<PathBuf>,
    sample_rate: u32,
    bits_per_sample: u32,
    channels: u32,
    compression_level: u32,
}

pub struct Encoder {
    stream_encoder: *mut FLAC__StreamEncoder,
}

fn get_state(stream_encoder: *mut FLAC__StreamEncoder) -> FLAC__StreamEncoderState {
    unsafe { FLAC__stream_encoder_get_state(stream_encoder) }
}

impl EncoderConfig {
    pub fn new() -> Self {
        EncoderConfig {
            output_path: None,
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 24,
            compression_level: 8,
        }
    }

    pub fn set_output_path(&mut self, output_path: &Path) {
        self.output_path = Some(output_path.into());
    }

    pub fn get_encoder(self) -> Result<Encoder, String> {
        let path = self
            .output_path
            .clone()
            .ok_or("Output path must be set".to_string())?;
        let output_path_cstring = CString::new(path.display().to_string())
            .expect("output path to be converted to CString");

        let stream_encoder = unsafe { FLAC__stream_encoder_new() };
        if stream_encoder.is_null() {
            return Err(
                "Could not initialize Flac stream encoder. Stream Encoder is null.".to_string(),
            );
        }

        // these can only fail if the stream encoder has already been initialized, but we don't do that until after this block
        unsafe {
            FLAC__stream_encoder_set_compression_level(stream_encoder, self.compression_level);
            FLAC__stream_encoder_set_channels(stream_encoder, self.channels);
            FLAC__stream_encoder_set_sample_rate(stream_encoder, self.sample_rate);
            FLAC__stream_encoder_set_bits_per_sample(stream_encoder, self.bits_per_sample);
        }

        let init_status = unsafe {
            FLAC__stream_encoder_init_file(
                stream_encoder,
                output_path_cstring.as_ptr(),
                None,
                ptr::null_mut(),
            )
        };

        if init_status != FLAC__StreamEncoderInitStatus_FLAC__STREAM_ENCODER_INIT_STATUS_OK {
            return Err(format!(
                "Stream Encoder file initialization failed. Status: {:?}",
                init_status
            ));
        }

        Ok(Encoder { stream_encoder })
    }
}

impl Encoder {
    pub fn encode(&self, data: &Vec<i32>) -> Result<(), String> {
        let success = unsafe {
            FLAC__stream_encoder_process(self.stream_encoder, &data.as_ptr(), data.len() as u32)
        } != 0;

        if success {
            Ok(())
        } else {
            let state = get_state(self.stream_encoder);
            Err(format!("Failed to encode. Encoder state: {}", state))
        }
    }

    // finish takes ownership of self and drops it since it will be invalid after this function
    pub fn finish(self) -> Result<(), String> {
        let success = unsafe { FLAC__stream_encoder_finish(self.stream_encoder) } != 0;

        if success {
            unsafe { FLAC__stream_encoder_delete(self.stream_encoder) };
            Ok(())
        } else {
            let state = get_state(self.stream_encoder);

            unsafe { FLAC__stream_encoder_delete(self.stream_encoder) };

            Err(format!(
                "Failed to finish encoding. Encoder state: {}",
                state
            ))
        }
    }
}
