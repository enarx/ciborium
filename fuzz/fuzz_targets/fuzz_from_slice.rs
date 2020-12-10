#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ciborium::de::from_reader::<ciborium::value::Value, _>(data);
});
