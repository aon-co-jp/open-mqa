//! Opus(Ogg Opus、RFC 6716/7845)エンコード/デコードの薄いラッパー。純Rust実装の`opus-pure`に委譲する
//! (C/FFI・cmake不要)。**非可逆**で、扱えるレートは8/12/16/24/48kHz・1〜2chのみ(ハイレゾ用ではない)。
//! DoP(DSD)は非可逆圧縮で壊れるため、Opusには載せない。

use crate::wav::DecodedAudio;
use opus_pure::{Application, OggOpusReader, OggOpusWriter, OpusEncoder, OpusHead, Trim, MAX_PACKET_BYTES, MAX_PACKET_SAMPLES};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpusError {
    #[error("Opusエラー: {0}")]
    Codec(String),
    #[error("Opusで扱えない入力です: {0}")]
    Unsupported(String),
}

fn e<T: std::fmt::Display>(x: T) -> OpusError {
    OpusError::Codec(x.to_string())
}

/// 整数PCMをOpusへエンコードする。`bits_per_sample`は入力サンプルのスケール(正規化用)。
pub fn encode_opus(samples_per_channel: &[Vec<i32>], sample_rate: u32, bits_per_sample: u8, bitrate_bps: i32) -> Result<Vec<u8>, OpusError> {
    let channels = samples_per_channel.len();
    if !(1..=2).contains(&channels) {
        return Err(OpusError::Unsupported(format!("{channels}chは非対応(1〜2ch)")));
    }
    if ![8000, 12000, 16000, 24000, 48000].contains(&sample_rate) {
        return Err(OpusError::Unsupported(format!("{sample_rate}Hzは非対応(8/12/16/24/48kHzのみ。先にリサンプルしてください)")));
    }
    let total = samples_per_channel[0].len();
    if samples_per_channel.iter().any(|c| c.len() != total) {
        return Err(OpusError::Unsupported("チャンネル間でサンプル数が違います".into()));
    }
    let scale = 1.0 / (1u64 << (bits_per_sample - 1)) as f32;
    let rate = sample_rate as i32;
    let frame = (rate / 50) as usize;
    let mut enc = OpusEncoder::new(rate, channels, Application::Audio).map_err(e)?;
    enc.bitrate_bps = bitrate_bps;
    let head = OpusHead::for_encoder(&enc, sample_rate);
    let ticks = 48_000 / sample_rate as usize;
    let frames = (total + (head.pre_skip as usize).div_ceil(ticks)).div_ceil(frame);
    let final_granule = u64::from(head.pre_skip) + (total * ticks) as u64;
    let mut writer = OggOpusWriter::new(Vec::new(), head).map_err(e)?;
    let mut packet = vec![0u8; MAX_PACKET_BYTES];
    let mut block = vec![0.0f32; frame * channels];
    for i in 0..frames {
        for f in 0..frame {
            for (c, ch) in samples_per_channel.iter().enumerate() {
                block[f * channels + c] = ch.get(i * frame + f).map_or(0.0, |&s| s as f32 * scale);
            }
        }
        let n = enc.encode(&block, frame, &mut packet).map_err(e)?;
        if i + 1 == frames {
            let duration = final_granule - writer.granule() as u64;
            writer.write_packet_with_duration(&packet[..n], duration as u32).map_err(e)?;
        } else {
            writer.write_packet(&packet[..n]).map_err(e)?;
        }
    }
    writer.finish().map_err(e)
}

/// Ogg Opusを16bit整数PCM(`sample_rate`で復号)へデコードする。
pub fn decode_opus(bytes: &[u8], sample_rate: u32) -> Result<DecodedAudio, OpusError> {
    let mut reader = OggOpusReader::new(std::io::Cursor::new(bytes)).map_err(e)?;
    let channels = reader.head().channel_count as usize;
    let rate = sample_rate as i32;
    let mut decoder = reader.head().decoder(rate).map_err(e)?;
    let mut trim = Trim::new(reader.head(), rate, channels).map_err(e)?;
    let mut block = vec![0.0f32; MAX_PACKET_SAMPLES * channels];
    let mut out = vec![Vec::new(); channels];
    for packet in reader.packets() {
        let packet = packet.map_err(e)?;
        let n = decoder.decode(&packet.data, MAX_PACKET_SAMPLES, &mut block).map_err(e)?;
        let kept = trim.keep(&packet, &block[..n * channels]);
        for fr in kept.chunks_exact(channels) {
            for (c, &s) in fr.iter().enumerate() {
                out[c].push((s * 32768.0).round().clamp(-32768.0, 32767.0) as i32);
            }
        }
    }
    Ok(DecodedAudio { sample_rate, bits_per_sample: 16, channels: channels as u32, samples_per_channel: out })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_round_trip_keeps_length_and_a_tone() {
        let n = 48_000;
        let tone: Vec<i32> = (0..n).map(|i| (12000.0 * (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / 48000.0).sin()) as i32).collect();
        let bytes = encode_opus(&[tone.clone(), tone.clone()], 48_000, 16, 128_000).unwrap();
        assert_eq!(&bytes[..4], b"OggS");
        let d = decode_opus(&bytes, 48_000).unwrap();
        assert_eq!((d.channels, d.num_frames()), (2, n), "ギャップレスで元と同じ長さになるはず");
        let dec = &d.samples_per_channel[0];
        let energy = |v: &[i32]| v.iter().map(|&x| (x as f64).powi(2)).sum::<f64>();
        let ratio = energy(&dec[4800..n - 4800]) / energy(&tone[4800..n - 4800]);
        assert!((0.8..1.2).contains(&ratio), "エネルギー比 {ratio}");
    }

    #[test]
    fn rejects_hires_rates_and_surround() {
        assert!(encode_opus(&[vec![0; 100]], 96_000, 24, 96_000).is_err());
        assert!(encode_opus(&[vec![0; 100], vec![0; 100], vec![0; 100]], 48_000, 16, 96_000).is_err());
    }
}
