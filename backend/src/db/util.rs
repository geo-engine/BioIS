use anyhow::Context;
use anyhow::Result;
use toasty::stmt::Value;

pub trait RecordValueExt {
    fn get_string(&self, idx: usize) -> Result<String>;
    fn get_string_list(&self, idx: usize) -> Result<Vec<String>>;
    fn get_string_list_option(&self, idx: usize) -> Result<Option<Vec<String>>>;
    fn get_number(&self, idx: usize) -> Result<f64>;
    fn get_number_option(&self, idx: usize) -> Result<Option<f64>>;
    fn get_bool(&self, idx: usize) -> Result<bool>;
}

impl RecordValueExt for Value {
    fn get_string(&self, idx: usize) -> Result<String> {
        value_to_string(record_value_at_index(self, idx)?)
    }

    fn get_string_list(&self, idx: usize) -> Result<Vec<String>> {
        list_value_to_string_list(record_value_at_index(self, idx)?)
    }

    fn get_string_list_option(&self, idx: usize) -> Result<Option<Vec<String>>> {
        let value = record_value_at_index(self, idx)?;
        if value.is_null() {
            return Ok(None);
        }

        list_value_to_string_list(value).map(Some)
    }

    fn get_number(&self, idx: usize) -> Result<f64> {
        value_to_number(record_value_at_index(self, idx)?)
    }

    fn get_number_option(&self, idx: usize) -> Result<Option<f64>> {
        let value = record_value_at_index(self, idx)?;
        if value.is_null() {
            return Ok(None);
        }
        value_to_number(value).map(Some)
    }

    fn get_bool(&self, idx: usize) -> Result<bool> {
        match record_value_at_index(self, idx)? {
            Value::Bool(b) => Ok(*b),
            _ => anyhow::bail!("expected bool value at index {idx}"),
        }
    }
}

#[inline]
fn value_to_string(value: &Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => anyhow::bail!("expected string value"),
    }
}

#[inline]
fn list_value_to_string_list(value: &Value) -> Result<Vec<String>> {
    let Value::List(list) = value else {
        anyhow::bail!("expected list value");
    };

    let mut result = Vec::new();
    for item in list {
        result.push(value_to_string(item)?);
    }
    Ok(result)
}

#[inline]
fn value_to_number(value: &Value) -> Result<f64> {
    match value {
        Value::F64(n) => Ok(*n),
        _ => anyhow::bail!("expected number value"),
    }
}

#[inline]
fn record_value_at_index(value: &Value, idx: usize) -> Result<&Value> {
    let record = value.as_record().context("expected record value")?;
    let Some(value) = record.get(idx) else {
        anyhow::bail!("expected value at index {idx}");
    };
    Ok(value)
}
