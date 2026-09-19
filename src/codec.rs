//! 出力形式の選択。**既定はWAV**(無圧縮・最も単純で互換性が高い)。FLAC(可逆圧縮)とOpus(非可逆、
//! 低レート配信向け)は選択制で、Cargoフィーチャ`flac`/`opus`(既定で有効)で組み込む。

use crate::wav::{self, DecodedAudio};
use thiserror::Error;

/// FLACのフレームヘッダが表せる上限サンプルレート(仕様上の最大値。`vendor/flacenc`のパッチで384kHz等が通る)。
pub const FLAC_MAX_RATE_HZ: u32 = 655_350;
/// 使用しているFLACエンコーダ(`flacenc`)が扱えるビット深度の上限(FLAC仕様は32bitまでだがエンコーダが24bit)。
pub const FLAC_MAX_BITS: u8 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioFormat {
    #[default]
    Wav,
    Flac,
    /// 非可逆。`bitrate_bps`はビットレート(例: 128000)。
    Opus { bitrate_bps: i32 },
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
            AudioFormat::Opus { .. } => "opus",
        }
    }

    /// DoP(DSD over PCM)をビット完全に保てる形式か(WAV/FLACは可、Opusは非可逆なので不可)。
    pub fn is_lossless(&self) -> bool {
        !matches!(self, AudioFormat::Opus { .. })
    }
}

#[derive(Debug, Error)]
pub enum CodecError {
    #[error(transparent)]
    Wav(#[from] wav::WavError),
    #[cfg(feature = "flac")]
    #[error(transparent)]
    Flac(#[from] crate::flac::FlacError),
    #[cfg(feature = "opus")]
    #[error(transparent)]
    Opus(#[from] crate::opus::OpusError),
    #[allow(dead_code)]
    #[error("この形式はビルドで無効です(Cargoフィーチャ`{0}`を有効にしてください)")]
    FeatureDisabled(&'static str),
    #[error("{0}")]
    Unsupported(String),
}

/// 指定形式でエンコードする。
pub fn encode(format: AudioFormat, samples_per_channel: &[Vec<i32>], sample_rate: u32, bits_per_sample: u8) -> Result<Vec<u8>, CodecError> {
    match format {
        AudioFormat::Wav => Ok(wav::encode_wav(samples_per_channel, sample_rate, bits_per_sample)?),
        #[cfg(feature = "flac")]
        AudioFormat::Flac if bits_per_sample > FLAC_MAX_BITS => Err(CodecError::Unsupported(format!("FLACエンコーダは{FLAC_MAX_BITS}bitまで。{bits_per_sample}bitはWAVを使ってください"))),
        #[cfg(feature = "flac")]
        AudioFormat::Flac if sample_rate > FLAC_MAX_RATE_HZ => Err(CodecError::Unsupported(format!("FLACは{FLAC_MAX_RATE_HZ}Hzまで。{sample_rate}HzはWAVを使ってください"))),
        #[cfg(feature = "flac")]
        AudioFormat::Flac => Ok(crate::flac::encode_flac(samples_per_channel, sample_rate, bits_per_sample)?),
        #[cfg(not(feature = "flac"))]
        AudioFormat::Flac => Err(CodecError::FeatureDisabled("flac")),
        #[cfg(feature = "opus")]
        AudioFormat::Opus { bitrate_bps } => Ok(crate::opus::encode_opus(samples_per_channel, sample_rate, bits_per_sample, bitrate_bps)?),
        #[cfg(not(feature = "opus"))]
        AudioFormat::Opus { .. } => Err(CodecError::FeatureDisabled("opus")),
    }
}

/// 先頭のマジックバイトからWAV/FLAC/Opusを自動判別してデコードする(Opusは48kHzで復号)。
pub fn decode(bytes: &[u8]) -> Result<DecodedAudio, CodecError> {
    if bytes.starts_with(b"RIFF") {
        return Ok(wav::decode_wav(bytes)?);
    }
    if bytes.starts_with(b"fLaC") {
        #[cfg(feature = "flac")]
        return Ok(crate::flac::decode_flac(bytes)?);
        #[cfg(not(feature = "flac"))]
        return Err(CodecError::FeatureDisabled("flac"));
    }
    if bytes.starts_with(b"OggS") {
        #[cfg(feature = "opus")]
        return Ok(crate::opus::decode_opus(bytes, 48_000)?);
        #[cfg(not(feature = "opus"))]
        return Err(CodecError::FeatureDisabled("opus"));
    }
    Err(CodecError::Unsupported("WAV/FLAC/Opusのいずれでもありません".into()))
}

/// DoP(DSD)フレームを保存する。可逆形式のみ許可する(Opusは壊れるため拒否。FLACは655,350Hzまでで、DSD128(352.8kHz)まで可、DSD256以上はWAV専用)。
pub fn encode_dop(format: AudioFormat, channels: &[Vec<crate::dop::PcmFrame24>], pcm_rate: u32) -> Result<Vec<u8>, CodecError> {
    if !format.is_lossless() {
        return Err(CodecError::Unsupported("DoPは非可逆のOpusでは壊れるため保存できません(WAVを選んでください)".into()));
    }
    let samples: Vec<Vec<i32>> = channels.iter().map(|ch| ch.iter().map(|f| i32::from_be_bytes([f[0], f[1], f[2], 0]) >> 8).collect()).collect();
    encode(format, &samples, pcm_rate, 24)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dop::{pack_dop_frames, unpack_dop_frames, DopConfig};

    #[test]
    fn default_format_is_wav() {
        assert_eq!(AudioFormat::default(), AudioFormat::Wav);
        assert_eq!(AudioFormat::default().extension(), "wav");
    }

    #[test]
    fn wav_and_flac_are_lossless_round_trips_via_auto_detection() {
        let ch = vec![(0..4000).map(|i| ((i as f64 * 0.03).sin() * 1_000_000.0) as i32).collect::<Vec<_>>(); 2];
        for f in [AudioFormat::Wav, AudioFormat::Flac] {
            let bytes = encode(f, &ch, 96_000, 24).unwrap();
            let d = decode(&bytes).unwrap();
            assert_eq!(d.samples_per_channel, ch, "{f:?}");
        }
    }

    #[test]
    fn opus_is_selectable_and_auto_detected() {
        let ch = vec![(0..48_000).map(|i| ((i as f64 * 0.13).sin() * 9000.0) as i32).collect::<Vec<_>>()];
        let bytes = encode(AudioFormat::Opus { bitrate_bps: 96_000 }, &ch, 48_000, 16).unwrap();
        assert_eq!(decode(&bytes).unwrap().num_frames(), 48_000);
    }

    #[test]
    fn flac_handles_high_rates_and_dop_is_lossless_but_opus_is_refused() {
        // FLAC 192k〜705.6kHz/24bitの可逆往復(96kHz超が通ること)
        let hi: Vec<i32> = (0..3000).map(|i| ((i as f64 * 0.02).sin() * 8_000_000.0) as i32).collect();
        for rate in [192_000u32, 352_800, 384_000, 655_350] {
            let f = encode(AudioFormat::Flac, &[hi.clone(), hi.clone()], rate, 24).unwrap();
            let d = decode(&f).unwrap();
            assert_eq!(d.sample_rate, rate);
            assert_eq!(d.samples_per_channel[0], hi, "FLAC {rate}Hz/24bit");
        }
        assert!(encode(AudioFormat::Flac, &[hi.clone()], 705_600, 24).is_err(), "FLAC仕様の上限655,350Hz超は不可");
        assert!(encode(AudioFormat::Flac, &[hi.clone()], 384_000, 32).is_err(), "FLACエンコーダは24bitまで");
        // DoP: DSD128(352.8kHz)まではWAV・FLACともビット完全に往復する。DSD256(705.6kHz)はFLACの上限超でWAVのみ。Opusは拒否。
        let dsd: Vec<u8> = (0..4096u32).map(|i| (i.wrapping_mul(2654435761) >> 11) as u8).collect();
        let cfg = DopConfig { format: crate::dop::DsdFormat::DSD128, container_bits: 24 };
        let frames = pack_dop_frames(&dsd, &cfg).unwrap();
        for f in [AudioFormat::Wav, AudioFormat::Flac] {
            let bytes = encode_dop(f, &[frames.clone(), frames.clone()], cfg.format.dop_pcm_sample_rate_hz()).unwrap();
            let d = decode(&bytes).unwrap();
            assert_eq!(d.sample_rate, 352_800);
            let back: Vec<[u8; 3]> = d.samples_per_channel[0].iter().map(|&s| [(s >> 16) as u8, (s >> 8) as u8, s as u8]).collect();
            assert_eq!(unpack_dop_frames(&back).unwrap(), dsd, "{f:?}");
        }
        let cfg256 = DopConfig::dsd256_24bit();
        let f256 = pack_dop_frames(&dsd, &cfg256).unwrap();
        assert!(encode_dop(AudioFormat::Wav, &[f256.clone()], 705_600).is_ok());
        assert!(encode_dop(AudioFormat::Flac, &[f256], 705_600).is_err(), "DSD256のDoPはFLACに載らない");
        assert!(encode_dop(AudioFormat::Opus { bitrate_bps: 128_000 }, &[frames], 352_800).is_err());
    }
}
