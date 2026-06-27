use crate::data;

#[derive(serde::Deserialize)]
struct CitiesRequest {
    operation: String,
    lang:      String,
}

pub fn execute(input: &str) -> Result<String, String> {
    let req: CitiesRequest = match serde_json::from_str(input) {
        Ok(r)  => r,
        Err(e) => return Err(format!("JSON parse error: {e}")),
    };
    if req.operation != "list_cities" {
        return Err(format!(
            "operation mismatch: op_ptr=list_cities body={}",
            req.operation
        ));
    }
    Ok(data::list_cities(&req.lang))
}
