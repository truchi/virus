use std::time::Duration;
use tween::TweenValue;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Tween                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A tween.
#[derive(Copy, Clone, Default, Debug)]
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
//                                              Tweened                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`Tween`]ed property `T`.
#[derive(Copy, Clone, Default, Debug)]
pub struct Tweened<T: TweenValue> {
    start: T,
    end: T,
    current: T,
    time: Duration,
    duration: Duration,
    tween: Tween,
}

impl<T: TweenValue> Tweened<T> {
    /// Creates a new (not) `Tweened` at `current`.
    pub fn new(current: T) -> Self {
        Self {
            start: current,
            end: current,
            current,
            time: Duration::ZERO,
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

    /// Returns the delta between the start and the end values.
    pub fn delta(&self) -> T {
        self.end - self.start
    }

    /// Returns the current value.
    pub fn current(&self) -> T {
        self.current
    }

    /// Returns the current time.
    pub fn time(&self) -> Duration {
        self.time
    }

    /// Returns the total duration of the animation.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the current time, normalized.
    pub fn percent(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    /// Returns the tween.
    pub fn tween(&self) -> Tween {
        self.tween
    }

    /// Returns `true` if animating, `false` otherwise.
    pub fn is_animating(&self) -> bool {
        self.time < self.duration
    }

    /// Tweens from `self.current()` to `end` for `duration` with `tween`.
    pub fn to(&mut self, end: T, duration: Duration, tween: Tween) {
        *self = Self {
            start: self.current,
            end,
            current: self.current,
            time: Duration::ZERO,
            duration,
            tween,
        }
    }

    /// Steps the animation by `delta` time and returns the current value.
    pub fn step(&mut self, delta: Duration) -> T {
        self.time += delta;

        if self.time >= self.duration {
            self.current = self.end;
            self.time = self.duration;
        } else {
            self.current = self.start + self.tween.tween(self.delta(), self.percent());
        }

        self.current
    }
}
