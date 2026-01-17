use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

static FIELDS: &[&str] = &["day", "condition", "high", "low"];
fn numeric_field() -> Option<String> { Some("high".to_string()) }
fn store_path() -> &'static str { "data/store.txt" }

fn parse_kv(items: &[String]) -> Result<HashMap<String, String>, String> {
    let mut record = HashMap::new();
    for item in items {
        let parts: Vec<&str> = item.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("invalid item: {item}"));
        }
        let key = parts[0].to_string();
        let value = parts[1].to_string();
        if !FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown field: {key}"));
        }
        if value.contains('|') {
            return Err("value may not contain '|'".to_string());
        }
        record.insert(key, value);
    }
    for f in FIELDS {
        record.entry((*f).to_string()).or_insert_with(String::new);
    }
    Ok(record)
}

fn format_record(values: &HashMap<String, String>) -> String {
    let mut parts = Vec::new();
    for k in FIELDS {
        let v = values.get(&k.to_string()).cloned().unwrap_or_default();
        parts.push(format!("{k}={v}"));
    }
    parts.join("|")
}

fn parse_line(line: &str) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    for part in line.trim().split('|') {
        if part.is_empty() { continue; }
        let kv: Vec<&str> = part.splitn(2, '=').collect();
        if kv.len() != 2 {
            return Err(format!("bad part: {part}"));
        }
        values.insert(kv[0].to_string(), kv[1].to_string());
    }
    Ok(values)
}

fn load_records() -> Result<Vec<HashMap<String, String>>, String> {
    if !Path::new(store_path()).exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(store_path()).map_err(|e| e.to_string())?;
    let mut records = Vec::new();
    for line in data.lines() {
        if line.trim().is_empty() { continue; }
        records.push(parse_line(line)?);
    }
    Ok(records)
}

fn append_record(values: &HashMap<String, String>) -> Result<(), String> {
    if let Some(dir) = Path::new(store_path()).parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut f = fs::OpenOptions::new().create(true).append(true).open(store_path())
        .map_err(|e| e.to_string())?;
    writeln!(f, "{}", format_record(values)).map_err(|e| e.to_string())?;
    Ok(())
}

fn summary(records: &[HashMap<String, String>]) -> String {
    let count = records.len();
    if let Some(field) = numeric_field() {
        let mut total: i64 = 0;
        for r in records {
            if let Some(v) = r.get(&field) {
                if let Ok(n) = v.parse::<i64>() { total += n; }
            }
        }
        return format!("count={count}, {field}_total={total}");
    }
    format!("count={count}")
}

fn main_inner() -> Result<(), String> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("Usage: init | add key=value... | list | summary");
        return Ok(());
    }
    let cmd = args.remove(0);
    match cmd.as_str() {
        "init" => {
            if let Some(dir) = Path::new(store_path()).parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            fs::write(store_path(), "").map_err(|e| e.to_string())?;
        }
        "add" => {
            let record = parse_kv(&args)?;
            append_record(&record)?;
        }
        "list" => {
            let records = load_records()?;
            for r in records {
                println!("{}", format_record(&r));
            }
        }
        "summary" => {
            let records = load_records()?;
            println!("{}", summary(&records));
        }
        _ => {
            return Err(format!("Unknown command: {cmd}"));
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_inner() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}
