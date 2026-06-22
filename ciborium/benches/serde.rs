// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::hint::black_box;
use std::io::Cursor;

use ciborium::{de::from_reader, from_slice, ser::into_writer};
use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct SampleDocument {
    version: u16,
    device: String,
    active: bool,
    tags: Vec<String>,
    metadata: BTreeMap<String, i64>,
    entries: Vec<SensorEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SensorEntry {
    id: u32,
    label: String,
    #[serde(with = "serde_bytes")]
    payload: Vec<u8>,
    readings: Vec<f64>,
    comment: Option<String>,
    states: [bool; 4],
}

#[derive(Serialize, Deserialize, Clone)]
struct SampleDocumentBorrow<'a> {
    version: u16,
    #[serde(borrow)]
    device: &'a str,
    active: bool,
    #[serde(borrow)]
    tags: Vec<&'a str>,
    metadata: BTreeMap<String, i64>,
    entries: Vec<SensorEntryBorrow<'a>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SensorEntryBorrow<'a> {
    id: u32,
    #[serde(borrow)]
    label: &'a str,
    payload: &'a [u8],
    readings: Vec<f64>,
    #[serde(borrow)]
    comment: Option<&'a str>,
    states: [bool; 4],
}

fn sample_document() -> SampleDocument {
    let mut metadata = BTreeMap::new();
    metadata.insert("alpha".into(), 2);
    metadata.insert("beta".into(), 8);
    metadata.insert("gamma".into(), 13);
    metadata.insert("delta".into(), 21);

    let entries = (0..32)
        .map(|idx| SensorEntry {
            id: idx,
            label: format!("sensor-{idx:02}"),
            payload: (0..64)
                .map(|offset| ((idx * 13 + offset) % 255) as u8)
                .collect(),
            readings: (0..8)
                .map(|offset| (idx * 100 + offset) as f64 * 0.125)
                .collect(),
            comment: if idx % 3 == 0 {
                Some(format!("reading {idx} is nominal"))
            } else {
                None
            },
            states: [
                idx % 2 == 0,
                idx % 3 == 0,
                idx % 5 == 0,
                idx % 7 == 0,
            ],
        })
        .collect();

    SampleDocument {
        version: 2,
        device: "env-sensor".into(),
        active: true,
        tags: vec!["bench".into(), "serde".into(), "cbor".into()],
        metadata,
        entries,
    }
}

fn serialize_benchmark(c: &mut Criterion) {
    let document = sample_document();
    let mut scratch = Vec::new();
    into_writer(&document, &mut scratch).expect("seed serialization");

    c.bench_function("into_writer", |b| {
        let mut buffer = Vec::with_capacity(scratch.len());
        b.iter(|| {
            buffer.clear();
            into_writer(black_box(&document), &mut buffer).expect("serialize document");
            black_box(buffer.len())
        });
    });
}

fn deserialize_benchmark(c: &mut Criterion) {
    let document = sample_document();
    let encoded = {
        let mut buffer = Vec::new();
        into_writer(&document, &mut buffer).expect("seed serialization");
        buffer
    };

    c.bench_function("from_reader", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(encoded.as_slice()));
            let decoded: SampleDocument =
                from_reader(cursor).expect("deserialize sample document");
            black_box(decoded.active)
        });
    });

    c.bench_function("from_slice", |b| {
        b.iter(|| {
            let decoded: SampleDocument =
                from_slice(black_box(encoded.as_slice())).expect("deserialize sample document");
            black_box(decoded.active)
        });
    });

    c.bench_function("from_slice_borrowed", |b| {
        b.iter(|| {
            let decoded: SampleDocumentBorrow<'_> =
                from_slice(black_box(encoded.as_slice())).expect("deserialize sample document");
            black_box(decoded.active)
        });
    });
}

criterion_group!(serde, serialize_benchmark, deserialize_benchmark);
criterion_main!(serde);
