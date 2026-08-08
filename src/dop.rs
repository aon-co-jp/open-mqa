//! DSD-over-PCM (DoP) のマーカー/フレームパッキング。
//!
//! DoP標準(AES/PMC-DSD、AES-51id-2012相当の業界慣行)は、1バイトのDSDデータ
//! 2個(16bit)と1バイトのマーカー(交互に`0x05`/`0xFA`)を組み合わせて
//! 24bit PCMコンテナへ格納する。マーカーバイトはMSB側(24bitのうち上位8bit)
//! に置き、DSDデータはその下位16bitへ収める、というのが一般的な実装
//! (対応DACはこのマーカー列を検出してDSD再生モードへ切り替える)。
//!
//! サンプルレート・ビット深度をハードコードしないという方針
//! (`dream-os/CLAUDE.md`「SOUND関連の技術提案」節)に従い、この実装では
//! DSD256(11.2896MHz)を含む任意のDSD世代のバイト列をパッキングできる
//! ようにパラメータ化している(DoPコンテナのPCM側サンプルレートは
//! DSDビットレートの1/16、これは仕様上固定の関係であり定数として扱う)。

use thiserror::Error;

/// DoPマーカーバイト(偶数フレームと奇数フレームで交互に使用)。
pub const DOP_MARKER_EVEN: u8 = 0x05;
pub const DOP_MARKER_ODD: u8 = 0xFA;

/// DoPコンテナの1PCMフレームあたりに収まるDSDビット数。
/// (24bitコンテナ - 8bitマーカー) = 16bit = DSDバイト2個分。
pub const DOP_DSD_BYTES_PER_FRAME: usize = 2;

/// サポートするDSD世代。サンプルレート・ビット深度をハードコードしない
/// 設計方針に沿い、値は列挙子ではなく構成パラメータとして保持する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdFormat {
    /// DSDのビットレート(Hz)。例: DSD256 = 11_289_600。
    pub dsd_bitrate_hz: u32,
}

impl DsdFormat {
    pub const DSD64: DsdFormat = DsdFormat {
        dsd_bitrate_hz: 2_822_400,
    };
    pub const DSD128: DsdFormat = DsdFormat {
        dsd_bitrate_hz: 5_644_800,
    };
    /// 2026-08-08時点の目標品質(dream-os/CLAUDE.md参照)。
    pub const DSD256: DsdFormat = DsdFormat {
        dsd_bitrate_hz: 11_289_600,
    };
    /// 2027年以降の普及を見据えた将来拡張(既に一部DAC実機は対応済み)。
    pub const DSD512: DsdFormat = DsdFormat {
        dsd_bitrate_hz: 22_579_200,
    };

    /// DoPコンテナ側のPCMサンプルレート(DoP仕様上、DSDビットレートの1/16)。
    pub fn dop_pcm_sample_rate_hz(&self) -> u32 {
        self.dsd_bitrate_hz / 16
    }
}

/// DoPパッキング設定。ビット深度(コンテナ幅)を24bit固定にせず、
/// 将来32bitコンテナへの拡張余地を持たせるためパラメータ化している
/// (現行DoP標準は24bitのみだが、ハードコード禁止方針に合わせる)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DopConfig {
    pub format: DsdFormat,
    /// PCMコンテナのビット深度。DoP標準は24。
    pub container_bits: u32,
}

impl DopConfig {
    pub fn dsd256_24bit() -> Self {
        DopConfig {
            format: DsdFormat::DSD256,
            container_bits: 24,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DopError {
    #[error("DSDバイト列の長さが2の倍数ではありません: {0}バイト")]
    OddByteLength(usize),
    #[error("container_bitsは24のみサポートしています(指定値: {0})")]
    UnsupportedContainerBits(u32),
    #[error("入力PCMフレーム数が不正です")]
    InvalidFrameCount,
}

/// 24bitコンテナへ詰めた1PCMフレームを表す(MSB→LSBの順で3バイト)。
pub type PcmFrame24 = [u8; 3];

/// DSDのバイト列(1チャンネル分、1バイト=8DSDビット)をDoPフレーム列へ
/// パッキングする。
///
/// `dsd_bytes`の長さは2の倍数である必要がある(1フレームにつきDSDバイト
/// 2個を消費するため)。戻り値の各要素は24bit PCMフレーム
/// (`[marker, dsd_byte_high, dsd_byte_low]`)。
pub fn pack_dop_frames(dsd_bytes: &[u8], config: &DopConfig) -> Result<Vec<PcmFrame24>, DopError> {
    if config.container_bits != 24 {
        return Err(DopError::UnsupportedContainerBits(config.container_bits));
    }
    if dsd_bytes.len() % DOP_DSD_BYTES_PER_FRAME != 0 {
        return Err(DopError::OddByteLength(dsd_bytes.len()));
    }

    let mut frames = Vec::with_capacity(dsd_bytes.len() / DOP_DSD_BYTES_PER_FRAME);
    for (i, chunk) in dsd_bytes.chunks_exact(DOP_DSD_BYTES_PER_FRAME).enumerate() {
        let marker = if i % 2 == 0 {
            DOP_MARKER_EVEN
        } else {
            DOP_MARKER_ODD
        };
        frames.push([marker, chunk[0], chunk[1]]);
    }

    Ok(frames)
}

/// [`pack_dop_frames`]の逆変換。DoPマーカーの並びを検証しつつ、DSD
/// バイト列を復元する。マーカーが交互パターン(0x05/0xFA)と一致しない
/// 場合はエラーとする(破損検知・DoP以外のPCMとの誤認防止のため)。
pub fn unpack_dop_frames(frames: &[PcmFrame24]) -> Result<Vec<u8>, DopError> {
    let mut dsd_bytes = Vec::with_capacity(frames.len() * DOP_DSD_BYTES_PER_FRAME);
    for (i, frame) in frames.iter().enumerate() {
        let expected_marker = if i % 2 == 0 {
            DOP_MARKER_EVEN
        } else {
            DOP_MARKER_ODD
        };
        if frame[0] != expected_marker {
            return Err(DopError::InvalidFrameCount);
        }
        dsd_bytes.push(frame[1]);
        dsd_bytes.push(frame[2]);
    }
    Ok(dsd_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsd256_bitrate_and_pcm_rate() {
        assert_eq!(DsdFormat::DSD256.dsd_bitrate_hz, 11_289_600);
        // DoPコンテナのPCMサンプルレートはDSDビットレートの1/16。
        assert_eq!(DsdFormat::DSD256.dop_pcm_sample_rate_hz(), 705_600);
    }

    #[test]
    fn dsd512_is_double_dsd256() {
        assert_eq!(
            DsdFormat::DSD512.dsd_bitrate_hz,
            DsdFormat::DSD256.dsd_bitrate_hz * 2
        );
    }

    #[test]
    fn pack_known_byte_pattern() {
        let config = DopConfig::dsd256_24bit();
        // 既知のDSDバイト列(4バイト = 2フレーム分)。
        let dsd_bytes = [0xAAu8, 0x55, 0x0F, 0xF0];
        let frames = pack_dop_frames(&dsd_bytes, &config).expect("pack failed");

        assert_eq!(frames.len(), 2);
        // 偶数フレーム(i=0)は0x05マーカー。
        assert_eq!(frames[0], [0x05, 0xAA, 0x55]);
        // 奇数フレーム(i=1)は0xFAマーカー。
        assert_eq!(frames[1], [0xFA, 0x0F, 0xF0]);
    }

    #[test]
    fn round_trip_pack_unpack() {
        let config = DopConfig::dsd256_24bit();
        let dsd_bytes: Vec<u8> = (0..=255u8).collect(); // 256バイト = 128フレーム
        let frames = pack_dop_frames(&dsd_bytes, &config).expect("pack failed");
        assert_eq!(frames.len(), 128);

        let recovered = unpack_dop_frames(&frames).expect("unpack failed");
        assert_eq!(recovered, dsd_bytes);
    }

    #[test]
    fn odd_length_input_rejected() {
        let config = DopConfig::dsd256_24bit();
        let dsd_bytes = [0x00u8, 0x01, 0x02]; // 3バイト(奇数)
        let result = pack_dop_frames(&dsd_bytes, &config);
        assert_eq!(result, Err(DopError::OddByteLength(3)));
    }

    #[test]
    fn unsupported_container_bits_rejected() {
        let config = DopConfig {
            format: DsdFormat::DSD256,
            container_bits: 32,
        };
        let result = pack_dop_frames(&[0x00, 0x01], &config);
        assert_eq!(result, Err(DopError::UnsupportedContainerBits(32)));
    }

    #[test]
    fn corrupted_marker_detected_on_unpack() {
        // 2フレーム目のマーカーが本来0xFAであるべきところを0x05のまま
        // 破損させたケース(マーカー交互パターン検証の確認)。
        let corrupted_frames: Vec<PcmFrame24> = vec![[0x05, 0xAA, 0x55], [0x05, 0x0F, 0xF0]];
        let result = unpack_dop_frames(&corrupted_frames);
        assert_eq!(result, Err(DopError::InvalidFrameCount));
    }

    #[test]
    fn empty_input_produces_empty_output() {
        let config = DopConfig::dsd256_24bit();
        let frames = pack_dop_frames(&[], &config).expect("pack failed");
        assert!(frames.is_empty());
    }
}
