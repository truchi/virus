use std::time::Duration;
use tween::TweenValue;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Tween                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A tween.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Tween {
    #[default]
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Tween {
    pub fn tween<T: TweenValue>(&mut self, target: T, percent: f32) -> T {
        match self {
            Tween::Linear => tween::Linear.tween(target, percent),
            Tween::SineIn => tween::SineIn.tween(target, percent),
            Tween::SineOut => tween::SineOut.tween(target, percent),
            Tween::SineInOut => tween::SineInOut.tween(target, percent),
            Tween::QuadIn => tween::QuadIn.tween(target, percent),
            Tween::QuadOut => tween::QuadOut.tween(target, percent),
            Tween::QuadInOut => tween::QuadInOut.tween(target, percent),
            Tween::CubicIn => tween::CubicIn.tween(target, percent),
            Tween::CubicOut => tween::CubicOut.tween(target, percent),
            Tween::CubicInOut => tween::CubicInOut.tween(target, percent),
            Tween::QuartIn => tween::QuartIn.tween(target, percent),
            Tween::QuartOut => tween::QuartOut.tween(target, percent),
            Tween::QuartInOut => tween::QuartInOut.tween(target, percent),
            Tween::QuintIn => tween::QuintIn.tween(target, percent),
            Tween::QuintOut => tween::QuintOut.tween(target, percent),
            Tween::QuintInOut => tween::QuintInOut.tween(target, percent),
            Tween::ExpoIn => tween::ExpoIn.tween(target, percent),
            Tween::ExpoOut => tween::ExpoOut.tween(target, percent),
            Tween::ExpoInOut => tween::ExpoInOut.tween(target, percent),
            Tween::CircIn => tween::CircIn.tween(target, percent),
            Tween::CircOut => tween::CircOut.tween(target, percent),
            Tween::CircInOut => tween::CircInOut.tween(target, percent),
            Tween::BackIn => tween::BackIn.tween(target, percent),
            Tween::BackOut => tween::BackOut.tween(target, percent),
            Tween::BackInOut => tween::BackInOut.tween(target, percent),
            Tween::ElasticIn => tween::ElasticIn.tween(target, percent),
            Tween::ElasticOut => tween::ElasticOut.tween(target, percent),
            Tween::ElasticInOut => tween::ElasticInOut.tween(target, percent),
            Tween::BounceIn => tween::BounceIn.tween(target, percent),
            Tween::BounceOut => tween::BounceOut.tween(target, percent),
            Tween::BounceInOut => tween::BounceInOut.tween(target, percent),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Tweenable                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Tweenable values.
pub trait Tweenable: TweenValue + Ord {}

impl<T: TweenValue + Ord> Tweenable for T {}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Tweened                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`Tween`]ed property `T`.
#[derive(Copy, Clone, Default, Debug)]
pub struct Tweened<T: Tweenable> {
    start: T,
    end: T,
    current: T,
    time: Option<Duration>,
    duration: Duration,
    tween: Tween,
}

impl<T: Tweenable> Tweened<T> {
    /// Creates a new `Tweened` at `current`, not animating.
    pub fn new(current: T) -> Self {
        Self {
            start: current,
            end: current,
            current,
            time: None,
            duration: Duration::ZERO,
            tween: Tween::Linear,
        }
    }

    /// Returns the start value.
    pub fn start(&self) -> T {
        self.start
    }

    /// Returns the end value.
    pub fn end(&self) -> T {
        self.end
    }

    /// Returns the current value.
    pub fn current(&self) -> T {
        self.current
    }

    /// Returns the current time.
    pub fn time(&self) -> Option<Duration> {
        self.time
    }

    /// Returns the total duration of the animation.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the tween.
    pub fn tween(&self) -> Tween {
        self.tween
    }

    /// Returns `true` if animating, `false` otherwise.
    pub fn is_animating(&self) -> bool {
        if self.duration.is_zero() {
            debug_assert!(self.current == self.start);
            debug_assert!(self.current == self.end);
            debug_assert!(self.time.is_none());
            false
        } else {
            self.time < Some(self.duration)
        }
    }

    /// Tweens from `self.current()` to `end` for `duration` with `tween`,
    /// or re-`new`s at end if `duration.is_zero()`.
    pub fn to(&mut self, end: T, duration: Duration, tween: Tween) {
        *self = if duration.is_zero() {
            Self::new(end)
        } else {
            Self {
                start: self.current,
                end,
                current: self.current,
                time: None,
                duration,
                tween,
            }
        };
    }

    /// Steps the animation by `delta` time and returns the current value.
    pub fn step(&mut self, delta: Duration) -> T {
        if self.duration.is_zero() {
            debug_assert!(self.current == self.start);
            debug_assert!(self.current == self.end);
            debug_assert!(self.time.is_none());
        } else if let Some(time) = self.time.map(|time| time + delta) {
            if time < self.duration {
                let percent = time.as_secs_f32() / self.duration.as_secs_f32();

                self.current = if self.start <= self.end {
                    self.start + self.tween.tween(self.end - self.start, percent)
                } else {
                    self.start - self.tween.tween(self.start - self.end, percent)
                };
                self.time = Some(time);
            } else {
                self.current = self.end;
                self.time = Some(self.duration);
            }
        } else {
            debug_assert!(self.current == self.start);
            self.time = Some(Duration::ZERO);
        }

        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_duration() {
        // Init
        let mut tweened = Tweened::new(0);
        assert!(tweened.start() == 0);
        assert!(tweened.end() == 0);
        assert!(tweened.current() == 0);
        assert!(tweened.time().is_none());
        assert!(tweened.duration().is_zero());
        assert!(tweened.tween() == Tween::Linear);
        assert!(!tweened.is_animating());

        // Not animating
        assert!(tweened.step(Duration::from_millis(999)) == 0);
        assert!(tweened.current() == 0);
        assert!(tweened.time().is_none());
        assert!(!tweened.is_animating());

        // Attempt to animate with Duration::ZERO, so renewed
        tweened.to(1, Duration::ZERO, Tween::Linear);
        assert!(tweened.start() == 1);
        assert!(tweened.end() == 1);
        assert!(tweened.current() == 1);
        assert!(tweened.time().is_none());
        assert!(tweened.duration().is_zero());
        assert!(tweened.tween() == Tween::Linear);
        assert!(!tweened.is_animating());

        // Still not animating
        assert!(tweened.step(Duration::from_millis(999)) == 1);
        assert!(tweened.current() == 1);
        assert!(tweened.time().is_none());
        assert!(!tweened.is_animating());
    }

    #[test]
    fn test() {
        // Init
        let mut tweened = Tweened::new(0);
        assert!(tweened.start() == 0);
        assert!(tweened.end() == 0);
        assert!(tweened.current() == 0);
        assert!(tweened.time().is_none());
        assert!(tweened.duration().is_zero());
        assert!(tweened.tween() == Tween::Linear);
        assert!(!tweened.is_animating());

        // Animate
        tweened.to(10, Duration::from_millis(10), Tween::Linear);
        assert!(tweened.start() == 0);
        assert!(tweened.end() == 10);
        assert!(tweened.current() == 0);
        assert!(tweened.time().is_none());
        assert!(tweened.duration() == Duration::from_millis(10));
        assert!(tweened.tween() == Tween::Linear);
        assert!(tweened.is_animating());

        // First frame
        assert!(tweened.step(Duration::from_millis(999)) == 0);
        assert!(tweened.current() == 0);
        assert!(tweened.time() == Some(Duration::ZERO));
        assert!(tweened.is_animating());

        // Second frame
        assert!(tweened.step(Duration::from_millis(1)) == 1);
        assert!(tweened.current() == 1);
        assert!(tweened.time() == Some(Duration::from_millis(1)));
        assert!(tweened.is_animating());

        // Nineth frame
        assert!(tweened.step(Duration::from_millis(8)) == 9);
        assert!(tweened.current() == 9);
        assert!(tweened.time() == Some(Duration::from_millis(9)));
        assert!(tweened.is_animating());

        // Last frame
        assert!(tweened.step(Duration::from_millis(1)) == 10);
        assert!(tweened.current() == 10);
        assert!(tweened.time() == Some(Duration::from_millis(10)));
        assert!(!tweened.is_animating());

        // Not animating anymore
        assert!(tweened.step(Duration::from_millis(1)) == 10);
        assert!(tweened.current() == 10);
        assert!(tweened.time() == Some(Duration::from_millis(10)));
        assert!(!tweened.is_animating());
    }
}
