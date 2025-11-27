mod drbg;

use crate::drbg::{AesCtrDrbg, Blake3XofDrbg, ChaCha20Drbg, Drbg};
use plotters::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const TARGET_LENGTHS: [usize; 4] = [10_000, 100_000, 1_000_000, 10_000_000];
const RUNS: usize = 50;
const BASE_SEED: &[u8] = b"cs-drbg-benchmark-seed-v1";

#[derive(Clone)]
struct Record {
    run: usize,
    generator: String,
    bits: usize,
    duration_ms: f64,
    storage_bytes: usize,
    zeros: u64,
    ones: u64,
}

#[derive(Clone)]
struct Summary {
    generator: String,
    bits: usize,
    runs: usize,
    mean_time_ms: f64,
    std_time_ms: f64,
    mean_ones_ratio: f64,
    std_ones_ratio: f64,
    storage_bytes: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all("results/plots")?;

    let mut records = Vec::new();
    for run in 0..RUNS {
        for &bits in TARGET_LENGTHS.iter() {
            let seed = make_seed(run, bits);
            let mut generators = build_generators(&seed);
            for drbg in generators.iter_mut() {
                let start = Instant::now();
                let bitstring = drbg.generate_bits(bits);
                let duration_ms = start.elapsed().as_secs_f64() * 1_000.0;
                let tally = bitstring.count_bits();

                records.push(Record {
                    run,
                    generator: drbg.name().to_string(),
                    bits,
                    duration_ms,
                    storage_bytes: bitstring.storage_bytes(),
                    zeros: tally.zeros,
                    ones: tally.ones,
                });
            }
        }
    }

    write_csv(&records)?;
    let summaries = summarize(&records);
    write_summary_csv(&summaries)?;

    plot_summary_metric(
        &summaries,
        Path::new("results/plots/time_ms.png"),
        "Generation time",
        "Time (ms)",
        |s| s.mean_time_ms,
    )?;
    plot_summary_metric(
        &summaries,
        Path::new("results/plots/memory_bytes.png"),
        "Space consumption (packed bits)",
        "Bytes",
        |s| s.storage_bytes as f64,
    )?;
    plot_summary_metric(
        &summaries,
        Path::new("results/plots/ones_ratio.png"),
        "Proportion of ones",
        "Ones ratio",
        |s| s.mean_ones_ratio,
    )?;

    println!(
        "Wrote results to results/metrics.csv, results/summary.csv and plots to results/plots"
    );
    Ok(())
}

fn write_csv(records: &[Record]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("results/metrics.csv")?;
    writeln!(
        file,
        "run,generator,bits,duration_ms,storage_bytes,zeros,ones,ones_ratio"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{:.6},{},{},{},{:.6}",
            r.run,
            r.generator,
            r.bits,
            r.duration_ms,
            r.storage_bytes,
            r.zeros,
            r.ones,
            r.ones as f64 / r.bits as f64
        )?;
    }
    Ok(())
}

fn write_summary_csv(summaries: &[Summary]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("results/summary.csv")?;
    writeln!(
        file,
        "generator,bits,runs,mean_time_ms,std_time_ms,mean_ones_ratio,std_ones_ratio,storage_bytes"
    )?;
    for s in summaries {
        writeln!(
            file,
            "{},{},{},{:.6},{:.6},{:.6},{:.6},{}",
            s.generator,
            s.bits,
            s.runs,
            s.mean_time_ms,
            s.std_time_ms,
            s.mean_ones_ratio,
            s.std_ones_ratio,
            s.storage_bytes
        )?;
    }
    Ok(())
}

fn summarize(records: &[Record]) -> Vec<Summary> {
    let mut grouped: BTreeMap<(String, usize), Vec<&Record>> = BTreeMap::new();
    for r in records {
        grouped
            .entry((r.generator.clone(), r.bits))
            .or_default()
            .push(r);
    }

    let mut summaries = Vec::new();
    for ((generator, bits), samples) in grouped {
        let runs = samples.len();
        let mean_time_ms = mean(samples.iter().map(|r| r.duration_ms));
        let std_time_ms = stddev(samples.iter().map(|r| r.duration_ms), mean_time_ms);
        let ratios: Vec<f64> = samples
            .iter()
            .map(|r| r.ones as f64 / r.bits as f64)
            .collect();
        let mean_ones_ratio = mean(ratios.iter().copied());
        let std_ones_ratio = stddev(ratios.iter().copied(), mean_ones_ratio);

        summaries.push(Summary {
            generator,
            bits,
            runs,
            mean_time_ms,
            std_time_ms,
            mean_ones_ratio,
            std_ones_ratio,
            storage_bytes: samples[0].storage_bytes,
        });
    }

    summaries
}

fn mean<I: Iterator<Item = f64>>(mut iter: I) -> f64 {
    let mut count = 0f64;
    let mut sum = 0f64;
    while let Some(v) = iter.next() {
        sum += v;
        count += 1.0;
    }
    if count == 0.0 { 0.0 } else { sum / count }
}

fn stddev<I: Iterator<Item = f64>>(iter: I, mean: f64) -> f64 {
    let mut count = 0f64;
    let mut acc = 0f64;
    for v in iter {
        count += 1.0;
        let diff = v - mean;
        acc += diff * diff;
    }
    if count <= 1.0 {
        0.0
    } else {
        (acc / (count - 1.0)).sqrt()
    }
}

fn build_generators(seed: &[u8]) -> Vec<Box<dyn Drbg>> {
    vec![
        Box::new(ChaCha20Drbg::new(seed)),
        Box::new(AesCtrDrbg::new(seed)),
        Box::new(Blake3XofDrbg::new(seed)),
    ]
}

fn make_seed(run: usize, bits: usize) -> Vec<u8> {
    let mut seed = Vec::with_capacity(BASE_SEED.len() + 16);
    seed.extend_from_slice(BASE_SEED);
    seed.extend_from_slice(&(run as u64).to_be_bytes());
    seed.extend_from_slice(&(bits as u64).to_be_bytes());
    seed
}

fn plot_summary_metric<F>(
    summaries: &[Summary],
    path: &Path,
    title: &str,
    y_label: &str,
    value: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&Summary) -> f64,
{
    if summaries.is_empty() {
        return Ok(());
    }

    let x_min = summaries.iter().map(|r| r.bits as u64).min().unwrap();
    let x_max = summaries.iter().map(|r| r.bits as u64).max().unwrap();
    let mut y_min = summaries.iter().map(|r| value(r)).fold(f64::MAX, f64::min);
    let mut y_max = summaries.iter().map(|r| value(r)).fold(f64::MIN, f64::max);
    if y_min == y_max {
        y_min = 0.0;
        y_max *= 1.1;
    }
    if y_min > 0.0 {
        y_min *= 0.9;
    }
    if y_max == 0.0 {
        y_max = 1.0;
    }

    let mut grouped: BTreeMap<&str, Vec<&Summary>> = BTreeMap::new();
    for r in summaries {
        grouped.entry(&r.generator).or_default().push(r);
    }
    for series in grouped.values_mut() {
        series.sort_by_key(|r| r.bits);
    }

    let root = BitMapBackend::new(path, (1200, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 26).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Bits")
        .y_desc(y_label)
        .label_style(("sans-serif", 16))
        .axis_desc_style(("sans-serif", 18))
        .draw()?;

    for (idx, (name, series)) in grouped.iter().enumerate() {
        let color = Palette99::pick(idx);
        let legend_color = color.to_rgba();
        chart
            .draw_series(LineSeries::new(
                series.iter().map(|r| (r.bits as u64, value(r))),
                color.stroke_width(3),
            ))?
            .label(*name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], legend_color));

        chart.draw_series(
            series
                .iter()
                .map(|r| Circle::new((r.bits as u64, value(r)), 4, color.filled())),
        )?;
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 16))
        .draw()?;

    root.present()?;
    Ok(())
}
