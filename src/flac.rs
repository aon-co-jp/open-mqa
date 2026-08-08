//! FLACエンコード/デコードの薄いラッパー。
//!
//! コーデックの内部処理(予測・エントロピー符号化等)は自前実装せず、
//! 既存の実績あるcrateへ委譲する:
//! - デコード: [`claxon`](https://docs.rs/claxon)
//! - エンコード: [`flacenc`](https://docs.rs/flacenc)

use flacenc::component::BitRepr;
use flacenc::error::Verify;
use std::io::Cursor;
use thiserror::Error;

/// デコード結果。チャンネルはインターリーブせず、チャンネルごとの
/// サンプル列を保持する(呼び出し側でインターリーブ/非インターリーブを
/// 選べるようにするため)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub bits_per_sample: u32,
    pub channels: u32,
    /// チャンネル毎のサンプル列(`channels`本)。
    pub samples_per_channel: Vec<Vec<i32>>,
}

impl DecodedAudio {
    pub fn num_frames(&self) -> usize {
        self.samples_per_channel
            .first()
            .map(|c| c.len())
            .unwrap_or(0)
    }
}

#[derive(Debug, Error)]
pub enum FlacError {
    #[error("FLACデコードエラー: {0}")]
    Decode(String),
    #[error("FLACエンコードエラー: {0}")]
    Encode(String),
    #[error("入力データが不正です: {0}")]
    InvalidInput(String),
}

/// FLACバイト列をデコードし、チャンネル毎のPCMサンプル列を返す。
///
/// サンプルレート・ビット深度は入力ファイルのヘッダから読み取り、
/// ハードコードしない(DSD256等の高解像度ソースをダウンコンバートした
/// FLACであっても、そのままの値を保持して返す)。
pub fn decode_flac(bytes: &[u8]) -> Result<DecodedAudio, FlacError> {
    let cursor = Cursor::new(bytes);
    let mut reader =
        claxon::FlacReader::new(cursor).map_err(|e| FlacError::Decode(e.to_string()))?;

    let info = reader.streaminfo();
    let channels = info.channels;
    let bits_per_sample = info.bits_per_sample;
    let sample_rate = info.sample_rate;

    // STREAMINFOに記録された総フレーム数(チャンネル横断の1サンプル単位)。
    // エンコーダ実装によっては末尾ブロックをゼロ詰めして固定ブロック長を
    // 保つ場合があるため(本crateのflacenc利用箇所も該当)、ここで真の
    // 長さへ切り詰める。
    let total_frames = info.samples.map(|n| n as usize);

    let mut samples_per_channel: Vec<Vec<i32>> = vec![Vec::new(); channels as usize];

    let mut frame_reader = reader.samples();
    let mut ch_idx: usize = 0;
    while let Some(sample) = frame_reader.next() {
        let sample = sample.map_err(|e| FlacError::Decode(e.to_string()))?;
        samples_per_channel[ch_idx].push(sample);
        ch_idx = (ch_idx + 1) % channels as usize;
    }

    if let Some(total_frames) = total_frames {
        for ch in &mut samples_per_channel {
            ch.truncate(total_frames);
        }
    }

    Ok(DecodedAudio {
        sample_rate,
        bits_per_sample,
        channels,
        samples_per_channel,
    })
}

/// PCMサンプル(チャンネル毎、インターリーブされていない状態)をFLACへ
/// エンコードする。サンプルレート・ビット深度は引数で指定し、
/// ハードコードしない(将来DSD512等より高いレートへ対応する際も
/// このシグネチャのまま拡張できる)。
pub fn encode_flac(
    samples_per_channel: &[Vec<i32>],
    sample_rate: u32,
    bits_per_sample: u8,
) -> Result<Vec<u8>, FlacError> {
    let channels = samples_per_channel.len();
    if channels == 0 {
        return Err(FlacError::InvalidInput(
            "チャンネル数が0です".to_string(),
        ));
    }
    let num_frames = samples_per_channel[0].len();
    for (i, ch) in samples_per_channel.iter().enumerate() {
        if ch.len() != num_frames {
            return Err(FlacError::InvalidInput(format!(
                "チャンネル{}のサンプル数がチャンネル0と一致しません({} != {})",
                i,
                ch.len(),
                num_frames
            )));
        }
    }

    // flacenc::source::MemSource はインターリーブされたi32サンプル列を要求する。
    let mut interleaved: Vec<i32> = Vec::with_capacity(num_frames * channels);
    for frame in 0..num_frames {
        for ch in samples_per_channel {
            interleaved.push(ch[frame]);
        }
    }

    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|(_, e)| FlacError::Encode(format!("{:?}", e)))?;
    let source = flacenc::source::MemSource::from_samples(
        &interleaved,
        channels,
        bits_per_sample as usize,
        sample_rate as usize,
    );

    let block_size = config.block_size;
    let flac_stream = flacenc::encode_with_fixed_block_size(&config, source, block_size)
        .map_err(|e| FlacError::Encode(format!("{:?}", e)))?;

    let mut sink = flacenc::bitsink::ByteSink::new();
    flac_stream
        .write(&mut sink)
        .map_err(|e| FlacError::Encode(format!("{:?}", e)))?;

    Ok(sink.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_wave(num_frames: usize, sample_rate: u32, freq: f64, bits: u32) -> Vec<i32> {
        let max_amp = (1i64 << (bits - 1)) as f64 - 1.0;
        (0..num_frames)
            .map(|n| {
                let t = n as f64 / sample_rate as f64;
                let v = (2.0 * std::f64::consts::PI * freq * t).sin() * max_amp * 0.5;
                v.round() as i32
            })
            .collect()
    }

    #[test]
    fn round_trip_mono_sine_16bit() {
        let sample_rate = 44_100u32;
        let bits = 16u32;
        let mono = sine_wave(4_410, sample_rate, 440.0, bits);
        let channels = vec![mono.clone()];

        let encoded = encode_flac(&channels, sample_rate, bits as u8).expect("encode failed");
        assert!(!encoded.is_empty());
        // FLACストリームマーカー "fLaC"
        assert_eq!(&encoded[0..4], b"fLaC");

        let decoded = decode_flac(&encoded).expect("decode failed");
        assert_eq!(decoded.sample_rate, sample_rate);
        assert_eq!(decoded.channels, 1);
        assert_eq!(decoded.bits_per_sample, bits);
        assert_eq!(decoded.num_frames(), mono.len());
        assert_eq!(decoded.samples_per_channel[0], mono);
    }

    #[test]
    fn round_trip_stereo_sine_24bit() {
        let sample_rate = 48_000u32;
        let bits = 24u32;
        let left = sine_wave(2_000, sample_rate, 220.0, bits);
        let right = sine_wave(2_000, sample_rate, 330.0, bits);
        let channels = vec![left.clone(), right.clone()];

        let encoded = encode_flac(&channels, sample_rate, bits as u8).expect("encode failed");
        let decoded = decode_flac(&encoded).expect("decode failed");

        assert_eq!(decoded.channels, 2);
        assert_eq!(decoded.sample_rate, sample_rate);
        assert_eq!(decoded.samples_per_channel[0], left);
        assert_eq!(decoded.samples_per_channel[1], right);
    }

    #[test]
    fn mismatched_channel_lengths_rejected() {
        let channels = vec![vec![0, 1, 2], vec![0, 1]];
        let result = encode_flac(&channels, 44_100, 16);
        assert!(result.is_err());
    }
}
