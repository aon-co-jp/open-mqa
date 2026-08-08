//! PCM→DSD(1bit delta-sigma変調)の実装。
//!
//! [`dop`](crate::dop)モジュールのテストはこれまで既知のバイトパターン
//! (`0xAA, 0x55, ...`)や連番バイト列による往復一致検証に留まっていた
//! (2026-08-08 HANDOFF「次にすべきこと(3)」参照)——これらはDoPパッキング
//! ロジック自体の正しさは検証するが、「実際の音声信号から生成された
//! DSDビットストリーム」を通したE2E検証ではなかった。本モジュールは
//! 1次のΔΣ(delta-sigma)変調器を実装し、PCM信号(f32サンプル列)から
//! 本物の1bit DSDビットストリームを生成できるようにする。
//!
//! ΔΣ変調自体はDSDの標準的な生成方式であり、ここでは高次NTF
//! (ノイズシェーピング多項式)のような高品質化は行わない、最小限の
//! 1次積分器+1bit量子化器のみを実装する(過剰実装を避ける方針)。

/// 1次delta-sigma変調器。入力はオーバーサンプリング済みのPCMサンプル
/// (範囲は概ね[-1.0, 1.0])で、1サンプルにつき1bitのDSD出力を生成する。
///
/// 呼び出し側は、目的のDSDビットレート(例: [`crate::dop::DsdFormat::DSD256`]
/// の`11_289_600`Hz)と同じレートでサンプルを供給する必要がある——本実装は
/// リサンプリング/オーバーサンプリング自体は行わない(スコープ外)。
pub struct DeltaSigmaModulator {
    integrator: f32,
}

impl Default for DeltaSigmaModulator {
    fn default() -> Self {
        DeltaSigmaModulator { integrator: 0.0 }
    }
}

impl DeltaSigmaModulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 1サンプル分を変調し、1bit(true=DSDの`1`、false=DSDの`0`)を返す。
    fn step(&mut self, sample: f32) -> bool {
        // 量子化器の出力を+1.0/-1.0とみなし、その誤差(sample - output)を
        // 積分器へフィードバックする、標準的な1次ΔΣのブロック図。
        let output_bit = self.integrator >= 0.0;
        let output_level = if output_bit { 1.0 } else { -1.0 };
        self.integrator += sample - output_level;
        output_bit
    }

    /// PCMサンプル列全体を変調し、DSDビット列をMSBファーストで
    /// バイトへパッキングして返す(1バイト=8サンプル分)。
    /// 入力長が8の倍数でない場合、末尾は`0`(DSDの`0`ビット)で
    /// パディングする。
    pub fn modulate_to_bytes(&mut self, pcm: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((pcm.len() + 7) / 8);
        let mut current_byte = 0u8;
        let mut bit_count = 0u8;
        for &sample in pcm {
            let bit = self.step(sample);
            current_byte <<= 1;
            if bit {
                current_byte |= 1;
            }
            bit_count += 1;
            if bit_count == 8 {
                bytes.push(current_byte);
                current_byte = 0;
                bit_count = 0;
            }
        }
        if bit_count > 0 {
            current_byte <<= 8 - bit_count;
            bytes.push(current_byte);
        }
        bytes
    }
}

/// DSDバイト列(MSBファースト、[`DeltaSigmaModulator::modulate_to_bytes`]と
/// 同じビット順)を、単純な移動平均ローパスで概算PCMへ復調する
/// (`window`サンプル幅の矩形窓平均、+1.0/-1.0の平均を取るだけの
/// 最小実装——本物のDACのようなマルチビットノイズシェーピング復調
/// フィルタではない。E2E検証で「元信号との相関が高い」ことを確認する
/// 目的に限定したテスト用ユーティリティ)。
pub fn crude_demodulate(dsd_bytes: &[u8], window: usize) -> Vec<f32> {
    let mut bits: Vec<f32> = Vec::with_capacity(dsd_bytes.len() * 8);
    for &byte in dsd_bytes {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            bits.push(if bit == 1 { 1.0 } else { -1.0 });
        }
    }
    if window == 0 || bits.is_empty() {
        return Vec::new();
    }
    bits.windows(window)
        .step_by(window)
        .map(|w| w.iter().sum::<f32>() / w.len() as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dop::{pack_dop_frames, unpack_dop_frames, DopConfig};
    use std::f32::consts::PI;

    /// サイン波を生成する(振幅0.8、DCオフセット無し)。
    fn sine_wave(freq_hz: f32, sample_rate_hz: f32, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate_hz;
                0.8 * (2.0 * PI * freq_hz * t).sin()
            })
            .collect()
    }

    #[test]
    fn modulator_output_is_not_constant_for_varying_input() {
        // 1次ΔΣが入力に応じて実際に1/0を切り替えていること(定数出力に
        // 縮退していないこと)を確認する最小のサニティチェック。
        let pcm = sine_wave(1000.0, 48_000.0, 256);
        let mut modulator = DeltaSigmaModulator::new();
        let bytes = modulator.modulate_to_bytes(&pcm);
        assert_eq!(bytes.len(), 32); // 256サンプル / 8 = 32バイト
        assert!(bytes.iter().any(|&b| b != 0x00));
        assert!(bytes.iter().any(|&b| b != 0xFF));
    }

    #[test]
    fn real_dsd_bitstream_round_trips_through_dop_packing() {
        // 2026-08-08 HANDOFF「次にすべきこと(3)」対応: 既知バイトパターン
        // ではなく、実際にPCMサイン波から生成したDSDビットストリームを
        // DoPパッキング→アンパッキングし、バイト単位で完全一致することを
        // 確認する(DoP層はDSDの中身に一切関知しないため理論上当然だが、
        // 「本物のDSDデータ」を通した経路として実証する)。
        let sample_rate = DopConfig::dsd256_24bit().format.dsd_bitrate_hz as f32;
        let pcm = sine_wave(1_000.0, sample_rate, 8 * 4096); // 4096バイト分(偶数フレーム数を確保)
        let mut modulator = DeltaSigmaModulator::new();
        let dsd_bytes = modulator.modulate_to_bytes(&pcm);
        assert_eq!(dsd_bytes.len(), 4096);

        let config = DopConfig::dsd256_24bit();
        let frames = pack_dop_frames(&dsd_bytes, &config).expect("pack failed");
        assert_eq!(frames.len(), dsd_bytes.len() / 2);

        let recovered = unpack_dop_frames(&frames).expect("unpack failed");
        assert_eq!(recovered, dsd_bytes, "DoP往復後、実DSDビットストリームが完全一致しない");
    }

    #[test]
    fn demodulated_signal_correlates_with_original_pcm() {
        // 上記の往復一致だけでは「DoP層が中身を破壊していないか」しか
        // 検証できないため、加えて「そもそも変調後のDSDビット列が元の
        // 信号を実際に表現しているか」を、簡易復調→相関係数で確認する。
        let sample_rate = 2_822_400.0f32; // DSD64相当(計算量を抑えるため)
        let freq = 1_000.0;
        let num_samples = 8 * 4096;
        let pcm = sine_wave(freq, sample_rate, num_samples);

        let mut modulator = DeltaSigmaModulator::new();
        let dsd_bytes = modulator.modulate_to_bytes(&pcm);

        // オーバーサンプリング比(sample_rate / 元信号の実効帯域)程度の
        // 窓で移動平均を取り、元のサンプルレートへ概算ダウンサンプルする。
        let window = 32;
        let demodulated = crude_demodulate(&dsd_bytes, window);
        assert!(!demodulated.is_empty());

        // 元のPCM信号も同じ間引きで整列させ、ピアソン相関係数を計算する。
        let reference: Vec<f32> = pcm
            .chunks(window)
            .map(|w| w.iter().sum::<f32>() / w.len() as f32)
            .collect();
        let n = demodulated.len().min(reference.len());
        let (a, b) = (&demodulated[..n], &reference[..n]);

        let mean_a = a.iter().sum::<f32>() / n as f32;
        let mean_b = b.iter().sum::<f32>() / n as f32;
        let mut cov = 0.0f64;
        let mut var_a = 0.0f64;
        let mut var_b = 0.0f64;
        for i in 0..n {
            let da = (a[i] - mean_a) as f64;
            let db = (b[i] - mean_b) as f64;
            cov += da * db;
            var_a += da * da;
            var_b += db * db;
        }
        let correlation = cov / (var_a.sqrt() * var_b.sqrt());

        // 1次ΔΣ+粗い移動平均復調という最小構成でも、元信号との相関は
        // 高い値(実測でおおむね0.9以上)になるはず——弱い相関(実装が
        // 実質ノイズしか出していない場合)ではないことの実証。
        assert!(
            correlation > 0.7,
            "復調信号と元PCMの相関が低すぎる(相関={correlation}) — \
             DSD変調が元信号を実際に表現できていない可能性がある"
        );
    }
}
