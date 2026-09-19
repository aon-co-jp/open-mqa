//! WAV(RIFF/WAVE、リニアPCM)の読み書き。外部crate非依存の自前実装。
//!
//! FLACはやめ、WAVを中間・出力形式の基本とする(2026-09-19方針変更)。理由: 無圧縮で仕様が
//! 単純、`make-disk`(DSD/ハイレゾ変換)と同じ形式で受け渡せ、DoPも24bit PCMのWAVとして
//! そのまま格納できる。サンプルはビット深度16/24/32のリニアPCM(i32に符号拡張して保持)。
//! 3ch以上・24bit超は`WAVE_FORMAT_EXTENSIBLE`(0xFFFE)で書き出す。読み込みは両形式に対応。
//! データチャンク以外(LIST等)は読み飛ばす。

use thiserror::Error;

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
        self.samples_per_channel.first().map(|c| c.len()).unwrap_or(0)
    }
}

#[derive(Debug, Error)]
pub enum WavError {
    #[error("WAVの形式が不正です: {0}")]
    Format(String),
    #[error("入力データが不正です: {0}")]
    InvalidInput(String),
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

pub fn decode_wav(bytes: &[u8]) -> Result<DecodedAudio, WavError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavError::Format("RIFF/WAVEヘッダがありません".into()));
    }
    let mut pos = 12;
    let mut fmt: Option<(u16, u16, u32, u16)> = None; // (形式, ch, rate, bits)
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(bytes, pos + 4) as usize;
        let body = pos + 8;
        if id == b"fmt " {
            if size < 16 || body + size > bytes.len() {
                return Err(WavError::Format("fmtチャンクが短すぎます".into()));
            }
            let mut tag = u16_at(bytes, body);
            if tag == 0xFFFE && size >= 26 {
                tag = u16_at(bytes, body + 24); // SubFormatの先頭2バイト
            }
            fmt = Some((tag, u16_at(bytes, body + 2), u32_at(bytes, body + 4), u16_at(bytes, body + 14)));
        } else if id == b"data" {
            let (tag, ch, rate, bits) = fmt.ok_or_else(|| WavError::Format("fmtがdataより後にあります".into()))?;
            if tag != 1 {
                return Err(WavError::Format(format!("リニアPCM以外は未対応です(形式{tag})")));
            }
            if ![16, 24, 32].contains(&bits) || ch == 0 {
                return Err(WavError::Format(format!("未対応のビット深度/チャンネル数: {bits}bit {ch}ch")));
            }
            let end = (body + size).min(bytes.len());
            let data = &bytes[body..end];
            let bps = bits as usize / 8;
            let frame = bps * ch as usize;
            let n = data.len() / frame;
            let mut out = vec![Vec::with_capacity(n); ch as usize];
            for f in 0..n {
                for (c, o) in out.iter_mut().enumerate() {
                    let s = &data[f * frame + c * bps..][..bps];
                    o.push(match bps {
                        2 => i16::from_le_bytes([s[0], s[1]]) as i32,
                        3 => i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8,
                        _ => i32::from_le_bytes([s[0], s[1], s[2], s[3]]),
                    });
                }
            }
            return Ok(DecodedAudio { sample_rate: rate, bits_per_sample: bits as u32, channels: ch as u32, samples_per_channel: out });
        }
        pos = body + size + (size & 1);
    }
    Err(WavError::Format("dataチャンクがありません".into()))
}

pub fn encode_wav(samples_per_channel: &[Vec<i32>], sample_rate: u32, bits_per_sample: u8) -> Result<Vec<u8>, WavError> {
    let ch = samples_per_channel.len();
    if ch == 0 || ch > 65535 {
        return Err(WavError::InvalidInput("チャンネル数が不正です".into()));
    }
    if ![16u8, 24, 32].contains(&bits_per_sample) {
        return Err(WavError::InvalidInput(format!("未対応のビット深度: {bits_per_sample}")));
    }
    let n = samples_per_channel[0].len();
    if let Some((i, c)) = samples_per_channel.iter().enumerate().find(|(_, c)| c.len() != n) {
        return Err(WavError::InvalidInput(format!("チャンネル{i}のサンプル数が一致しません({} != {n})", c.len())));
    }
    let bps = bits_per_sample as usize / 8;
    let data_len = n * ch * bps;
    let extensible = ch > 2 || bits_per_sample > 16;
    let fmt_len = if extensible { 40 } else { 16 };
    let mut v = Vec::with_capacity(48 + data_len);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&((4 + 8 + fmt_len + 8 + data_len + (data_len & 1)) as u32).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&(fmt_len as u32).to_le_bytes());
    v.extend_from_slice(&(if extensible { 0xFFFEu16 } else { 1 }).to_le_bytes());
    v.extend_from_slice(&(ch as u16).to_le_bytes());
    v.extend_from_slice(&sample_rate.to_le_bytes());
    v.extend_from_slice(&(sample_rate * ch as u32 * bps as u32).to_le_bytes());
    v.extend_from_slice(&((ch * bps) as u16).to_le_bytes());
    v.extend_from_slice(&(bits_per_sample as u16).to_le_bytes());
    if extensible {
        v.extend_from_slice(&22u16.to_le_bytes());
        v.extend_from_slice(&(bits_per_sample as u16).to_le_bytes());
        v.extend_from_slice(&0u32.to_le_bytes()); // チャンネルマスク未指定
        v.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xAA, 0, 0x38, 0x9B, 0x71]); // PCMのGUID
    }
    v.extend_from_slice(b"data");
    v.extend_from_slice(&(data_len as u32).to_le_bytes());
    for f in 0..n {
        for c in samples_per_channel {
            v.extend_from_slice(&c[f].to_le_bytes()[..bps]);
        }
    }
    if data_len & 1 == 1 {
        v.push(0);
    }
    Ok(v)
}

/// DoPフレーム(24bit、`[マーカー, DSD上位バイト, DSD下位バイト]`)列を、DoP用PCMレート
/// (DSDレート/16)の24bit WAVとして書き出す。DoP対応DACはマーカー列からDSD再生へ切り替える。
/// **ビットパーフェクト再生が前提**(音量・SRC・ミキサーを通るとDSDが壊れノイズになる)。
pub fn encode_dop_wav(channels: &[Vec<crate::dop::PcmFrame24>], sample_rate: u32) -> Result<Vec<u8>, WavError> {
    let samples: Vec<Vec<i32>> =
        channels.iter().map(|ch| ch.iter().map(|f| i32::from_be_bytes([f[0], f[1], f[2], 0]) >> 8).collect()).collect();
    encode_wav(&samples, sample_rate, 24)
}

/// [`encode_dop_wav`]の逆。24bit WAVからDoPフレーム列(チャンネル毎)を取り出す。
pub fn decode_dop_wav(bytes: &[u8]) -> Result<Vec<Vec<crate::dop::PcmFrame24>>, WavError> {
    let d = decode_wav(bytes)?;
    if d.bits_per_sample != 24 {
        return Err(WavError::Format("DoPは24bit WAVが必要です".into()));
    }
    Ok(d.samples_per_channel
        .iter()
        .map(|ch| ch.iter().map(|&s| [(s >> 16) as u8, (s >> 8) as u8, s as u8]).collect())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dop::{pack_dop_frames, unpack_dop_frames, DopConfig};

    fn sine(n: usize, amp: f64) -> Vec<i32> {
        (0..n).map(|i| (amp * (i as f64 * 0.05).sin()) as i32).collect()
    }

    #[test]
    fn round_trips_16_24_and_32_bit_stereo_exactly() {
        for (bits, amp) in [(16u8, 30000.0), (24, 8_000_000.0), (32, 2_000_000_000.0)] {
            let ch = vec![sine(1001, amp), sine(1001, amp * 0.5)];
            let wav = encode_wav(&ch, 96_000, bits).unwrap();
            let d = decode_wav(&wav).unwrap();
            assert_eq!((d.sample_rate, d.bits_per_sample, d.channels), (96_000, bits as u32, 2));
            assert_eq!(d.samples_per_channel, ch, "{bits}bit");
        }
    }

    #[test]
    fn round_trips_six_channels() {
        let ch: Vec<Vec<i32>> = (0..6).map(|c| sine(333, 1000.0 * (c + 1) as f64)).collect();
        assert_eq!(decode_wav(&encode_wav(&ch, 48_000, 16).unwrap()).unwrap().samples_per_channel, ch);
    }

    #[test]
    fn negative_24bit_values_keep_their_sign() {
        let ch = vec![vec![-8_388_608, -1, 0, 1, 8_388_607]];
        assert_eq!(decode_wav(&encode_wav(&ch, 44_100, 24).unwrap()).unwrap().samples_per_channel, ch);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(decode_wav(b"not a wav file").is_err());
        assert!(encode_wav(&[], 44_100, 16).is_err());
        assert!(encode_wav(&[vec![0]], 44_100, 12).is_err());
        assert!(encode_wav(&[vec![0, 1], vec![0]], 44_100, 16).is_err());
    }

    #[test]
    fn skips_unknown_chunks_before_data() {
        let mut wav = encode_wav(&[vec![1, 2, 3]], 44_100, 16).unwrap();
        let data_pos = wav.windows(4).position(|w| w == b"data").unwrap();
        let mut list = b"LIST".to_vec();
        list.extend_from_slice(&3u32.to_le_bytes());
        list.extend_from_slice(&[9, 9, 9, 0]);
        for (i, b) in list.into_iter().enumerate() {
            wav.insert(data_pos + i, b);
        }
        assert_eq!(decode_wav(&wav).unwrap().samples_per_channel, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn dop_survives_a_wav_round_trip_bit_exactly_including_the_markers() {
        // 高いビットが立つバイト(0xFA等)を含む、DSDバイト列。
        let dsd: Vec<u8> = (0..4096u32).map(|i| (i.wrapping_mul(2654435761) >> 13) as u8).collect();
        let cfg = DopConfig::dsd256_24bit();
        let frames = pack_dop_frames(&dsd, &cfg).unwrap();
        let wav = encode_dop_wav(&[frames.clone(), frames], cfg.format.dop_pcm_sample_rate_hz()).unwrap();
        let back = decode_dop_wav(&wav).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(unpack_dop_frames(&back[0]).unwrap(), dsd);
        assert_eq!(decode_wav(&wav).unwrap().sample_rate, 705_600);
    }
}
