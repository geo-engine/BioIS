use toasty::stmt::Value;

pub fn get_string(value: &Value, idx: usize) -> Option<String> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

pub fn get_string_list(value: &Value, idx: usize) -> Option<Vec<String>> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::List(list)) => {
            let mut result = Vec::new();
            for item in list {
                if let Value::String(s) = item {
                    result.push(s.clone());
                } else {
                    return None;
                }
            }
            Some(result)
        }
        _ => None,
    }
}

pub fn get_string_list_option(value: &Value, idx: usize) -> Option<Option<Vec<String>>> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::List(list)) => {
            let mut result = Vec::new();
            for item in list {
                if let Value::String(s) = item {
                    result.push(s.clone());
                } else {
                    return None;
                }
            }
            Some(Some(result))
        }
        Some(Value::Null) => Some(None),
        _ => None,
    }
}

pub fn get_number(value: &Value, idx: usize) -> Option<f64> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::F64(n)) => Some(*n),
        _ => None,
    }
}

pub fn get_number_option(value: &Value, idx: usize) -> Option<Option<f64>> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::F64(n)) => Some(Some(*n)),
        Some(Value::Null) => Some(None),
        _ => None,
    }
}

pub fn get_bool(value: &Value, idx: usize) -> Option<bool> {
    let record = value.as_record()?;
    match record.get(idx) {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    }
}
