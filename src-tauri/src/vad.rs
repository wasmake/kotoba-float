use std::collections::VecDeque;

pub struct Segmenter {
    threshold: f32,
    trailing_samples: usize,
    max_samples: usize,
    pre_roll_samples: usize,
    silence_samples: usize,
    active: bool,
    pre: VecDeque<f32>,
    current: Vec<f32>,
}

impl Segmenter {
    pub fn new(sample_rate: u32, trailing_ms: u64, max_ms: u64, pre_roll_ms: u64) -> Self {
        Self {
            threshold: 0.012,
            trailing_samples: sample_rate as usize * trailing_ms as usize / 1000,
            max_samples: sample_rate as usize * max_ms as usize / 1000,
            pre_roll_samples: sample_rate as usize * pre_roll_ms as usize / 1000,
            silence_samples: 0,
            active: false,
            pre: VecDeque::new(),
            current: vec![],
        }
    }

    pub fn push(&mut self, frame: &[f32]) -> Option<Vec<f32>> {
        if frame.is_empty() {
            return None;
        }
        let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
        let speech = rms >= self.threshold;
        let mut started = false;
        if !self.active {
            self.pre.extend(frame.iter().copied());
            while self.pre.len() > self.pre_roll_samples.max(1) {
                self.pre.pop_front();
            }
            if !speech {
                return None;
            }
            self.active = true;
            started = true;
            self.current.extend(self.pre.drain(..));
        }
        if !started {
            self.current.extend_from_slice(frame);
        }
        self.silence_samples = if speech {
            0
        } else {
            self.silence_samples + frame.len()
        };
        if self.silence_samples >= self.trailing_samples.max(1)
            || self.current.len() >= self.max_samples
        {
            self.active = false;
            self.silence_samples = 0;
            return Some(std::mem::take(&mut self.current));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silence_never_creates_segment() {
        let mut v = Segmenter::new(16_000, 500, 6000, 200);
        for _ in 0..100 {
            assert!(v.push(&vec![0.0; 480]).is_none());
        }
    }
    #[test]
    fn trailing_silence_finalizes_once() {
        let mut v = Segmenter::new(16_000, 500, 6000, 200);
        assert!(v.push(&vec![0.2; 480]).is_none());
        let mut result = None;
        for _ in 0..17 {
            if let Some(s) = v.push(&vec![0.0; 480]) {
                result = Some(s);
                break;
            }
        }
        assert!(result.is_some());
    }
}
