//! Dashboard metrics derived from stored round records (`Scores`).
//! With fewer than two stored rounds, clearly sample data fills the charts so a
//! fresh install still shows the layout.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::storage::Scores;

const DAY: u64 = 86_400;
pub const DAILY_GOAL_HITS: u32 = 500;
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub struct DayStat {
    pub label: String,
    pub hits: u32,
    pub shots: u32,
}

impl DayStat {
    pub fn accuracy(&self) -> Option<f32> {
        (self.shots > 0).then(|| self.hits as f32 / self.shots as f32 * 100.0)
    }
}

pub struct DashStats {
    /// (unix_secs, score) of the last 28 rounds, oldest first.
    #[allow(dead_code)]
    pub series: Vec<(u64, f32)>,
    /// Last 7 days, oldest first.
    pub days: Vec<DayStat>,
    /// Hits per task, descending (top 4).
    pub task_hits: Vec<u32>,
    pub hits: u32,
    pub misses: u32,
    pub avg_ms_per_hit: Option<f32>,
    pub best_round_hits: u32,
    pub hits_today: u32,
    #[allow(dead_code)]
    pub sample: bool,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// "Jul 22" style label for a unix timestamp (UTC).
pub fn date_label(unix: u64) -> String {
    let z = (unix / DAY) as i64 + 719_468;
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{} {}", MONTHS[(month - 1) as usize], day)
}

impl DashStats {
    pub fn from_scores(scores: &Scores) -> Self {
        let now = now_secs();
        let today = now / DAY;

        let mut rounds: Vec<_> = scores.tasks.values().flat_map(|t| t.recent.iter()).collect();
        rounds.sort_by_key(|r| r.unix_secs);

        let mut task_hits: Vec<u32> = scores
            .tasks
            .values()
            .map(|t| t.recent.iter().map(|r| r.hits).sum())
            .filter(|h| *h > 0)
            .collect();
        task_hits.sort_unstable_by(|a, b| b.cmp(a));
        task_hits.truncate(4);

        let hits: u32 = rounds.iter().map(|r| r.hits).sum();
        let misses: u32 = rounds.iter().map(|r| r.misses).sum();
        let millis: f32 = rounds.iter().map(|r| r.duration as f32 * 1000.0).sum();

        let days: Vec<DayStat> = (0..7u64)
            .map(|i| {
                let day = today + i - 6;
                let (h, m) = rounds
                    .iter()
                    .filter(|r| r.unix_secs / DAY == day)
                    .fold((0, 0), |(h, m), r| (h + r.hits, m + r.misses));
                let full = date_label(day * DAY);
                let label = if i == 0 {
                    full
                } else {
                    full.split(' ').nth(1).unwrap_or("").to_string()
                };
                DayStat { label, hits: h, shots: h + m }
            })
            .collect();

        let best_round_hits = rounds.iter().map(|r| r.hits).max().unwrap_or(0);
        let hits_today = days.last().map(|d| d.hits).unwrap_or(0);

        if rounds.is_empty() {
            return Self::sample(now, days);
        }

        let skip = rounds.len().saturating_sub(28);
        let series = rounds.iter().skip(skip).map(|r| (r.unix_secs, r.score.max(0) as f32)).collect();
        Self {
            series,
            days,
            task_hits,
            hits,
            misses,
            avg_ms_per_hit: (hits > 0).then(|| millis / hits as f32),
            best_round_hits,
            hits_today,
            sample: false,
        }
    }

    fn sample(now: u64, mut days: Vec<DayStat>) -> Self {
        const BASE: [f32; 28] = [
            38e3, 42e3, 41e3, 46e3, 49e3, 53e3, 52e3, 56e3, 59e3, 58e3, 64e3, 68e3, 71e3, 76e3,
            73e3, 71e3, 75e3, 78e3, 74e3, 72e3, 79e3, 81e3, 78e3, 76e3, 80e3, 82e3, 79e3, 82.45e3,
        ];
        let hit = [700, 600, 810, 500, 650, 550, 870];
        let shots = [810, 700, 900, 540, 730, 640, 950];
        for (i, d) in days.iter_mut().enumerate() {
            d.hits = hit[i];
            d.shots = shots[i];
        }
        Self {
            series: BASE
                .iter()
                .enumerate()
                .map(|(i, v)| (now - (27 - i as u64) * DAY, *v))
                .collect(),
            days,
            task_hits: vec![350, 280, 220, 150],
            hits: 764,
            misses: 56,
            avg_ms_per_hit: Some(185.0),
            best_round_hits: 99,
            hits_today: 500,
            sample: true,
        }
    }

    pub fn accuracy_pct(&self) -> f32 {
        let shots = self.hits + self.misses;
        if shots == 0 { 0.0 } else { self.hits as f32 / shots as f32 * 100.0 }
    }
}
