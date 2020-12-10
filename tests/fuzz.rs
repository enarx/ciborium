#[test]
fn oom_1() {
    let data = vec![186, 197, 197, 197, 95];
    let ret = ciborium::de::from_reader::<ciborium::value::Value, _>(&data[..]);
    println!("{:#?}", ret);
}

#[test]
fn oom_2() {
    let data = vec![186, 94, 57, 57, 78, 11];
    let ret = ciborium::de::from_reader::<ciborium::value::Value, _>(&data[..]);
    println!("{:#?}", ret);
}

#[test]
fn oom_3() {
    let data = vec![155, 255, 255, 44, 89, 143, 143, 136, 136, 136];
    let ret = ciborium::de::from_reader::<ciborium::value::Value, _>(&data[..]);
    println!("{:#?}", ret);
}

#[test]
fn oom_4() {
    let data = vec![91, 20, 20, 20, 20, 20, 20, 20, 20];
    let ret = ciborium::de::from_reader::<ciborium::value::Value, _>(&data[..]);
    println!("{:#?}", ret);
}
