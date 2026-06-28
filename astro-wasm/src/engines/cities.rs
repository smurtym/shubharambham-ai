use crate::city_data;

#[derive(serde::Deserialize)]
struct CitiesRequest {
    operation: String,
    lang:      String,
}

pub fn execute(input: &str) -> String {
    let req: CitiesRequest = match serde_json::from_str(input) {
        Ok(r)  => r,
        Err(e) => {
            let msg = e.to_string().replace('"', "\\\"");
            return format!("{{\"error\":\"JSON parse error: {msg}\"}}");
        }
    };
    if req.operation != "list_cities" {
        let msg = format!("operation mismatch: op_ptr=list_cities body={}", req.operation)
            .replace('"', "\\\"");
        return format!("{{\"error\":\"{msg}\"}}");
    }
    city_data::list_cities(&req.lang)
}
