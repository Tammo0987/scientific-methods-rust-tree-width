use serde::Serialize;

#[derive(Default)]
pub struct Accumulator {
    total: usize,
    values: Vec<u32>,
    safe_values: Vec<u32>,
    unsafe_values: Vec<u32>,
}

impl Accumulator {
    pub fn add(&mut self, tw: u32, is_unsafe: bool) {
        self.total += 1;
        self.values.push(tw);
        if is_unsafe {
            self.unsafe_values.push(tw);
        } else {
            self.safe_values.push(tw);
        }
    }

    pub fn into_summary(mut self) -> Summary {
        self.values.sort_unstable();
        self.safe_values.sort_unstable();
        self.unsafe_values.sort_unstable();

        let distribution = {
            let max = self.values.last().copied().unwrap_or(0) as usize;
            (0..=max)
                .map(|tw| self.values.iter().filter(|&&x| x == tw as u32).count())
                .collect()
        };

        Summary {
            total: self.total,
            all: group_stats(&self.values),
            safe: group_stats(&self.safe_values),
            r#unsafe: group_stats(&self.unsafe_values),
            distribution,
        }
    }
}

#[derive(Serialize)]
pub struct Summary {
    pub total: usize,
    pub all: GroupStats,
    pub safe: GroupStats,
    pub r#unsafe: GroupStats,
    /// distribution[k] = number of functions with treewidth == k
    pub distribution: Vec<usize>,
}

#[derive(Serialize)]
pub struct GroupStats {
    pub count: usize,
    pub mean: f64,
    pub median: u32,
    pub max: u32,
    pub pct_le3: f64,
    pub pct_le6: f64,
}

fn group_stats(sorted: &[u32]) -> GroupStats {
    let count = sorted.len();
    if count == 0 {
        return GroupStats {
            count: 0,
            mean: 0.0,
            median: 0,
            max: 0,
            pct_le3: 0.0,
            pct_le6: 0.0,
        };
    }
    let mean = sorted.iter().map(|&x| x as f64).sum::<f64>() / count as f64;
    let median = sorted[count / 2];
    let max = *sorted.last().unwrap();
    let pct_le = |t: u32| 100.0 * sorted.iter().filter(|&&x| x <= t).count() as f64 / count as f64;
    GroupStats {
        count,
        mean,
        median,
        max,
        pct_le3: pct_le(3),
        pct_le6: pct_le(6),
    }
}

impl Summary {
    pub fn print(&self) {
        println!();
        println!("=== Treewidth Analysis Summary ===");
        println!();
        println!("Functions total:  {}", self.total);
        println!();
        println!("All solved:");
        print_group(&self.all);
        println!();
        println!("Safe ({} functions):", self.safe.count);
        print_group(&self.safe);
        println!();
        println!("Unsafe ({} functions):", self.r#unsafe.count);
        print_group(&self.r#unsafe);
        println!();

        println!("Distribution:");
        for (tw, &count) in self.distribution.iter().enumerate().take(15) {
            if count == 0 && tw > self.all.max as usize {
                break;
            }
            let bar_len = (count * 40 / self.total.max(1)).max(if count > 0 { 1 } else { 0 });
            println!("  tw={tw:2}: {count:6}  {}", "#".repeat(bar_len));
        }
        let overflow: usize = self.distribution.iter().skip(15).sum();
        if overflow > 0 {
            println!("  tw≥15: {overflow:6}");
        }
    }
}

fn print_group(g: &GroupStats) {
    println!(
        "  mean={:.2}  median={}  max={}  tw≤3={:.1}%  tw≤6={:.1}%",
        g.mean, g.median, g.max, g.pct_le3, g.pct_le6
    );
}
