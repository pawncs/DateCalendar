use serde::Serialize;
use std::io;

/// 输出结果，支持 json/table/csv 三种格式
pub fn output_result<T: Serialize>(data: &T, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(data)?;
            println!("{}", json);
        }
        "table" => {
            output_table(data)?;
        }
        "csv" => {
            output_csv(data)?;
        }
        _ => {
            return Err(format!("不支持的输出格式: {}", format).into());
        }
    }
    Ok(())
}

/// 输出为表格（使用对齐的文本格式）
fn output_table<T: Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json_value = serde_json::to_value(data)?;
    
    if let Some(array) = json_value.as_array() {
        if array.is_empty() {
            println!("(无数据)");
            return Ok(());
        }
        
        // 获取表头（从第一个元素）
        if let Some(first) = array.first() {
            if let Some(obj) = first.as_object() {
                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                
                // 计算每列最大宽度
                let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
                
                let rows: Vec<Vec<String>> = array.iter()
                    .filter_map(|item| {
                        if let Some(obj) = item.as_object() {
                            Some(headers.iter()
                                .map(|h| {
                                    let v = obj.get(*h).unwrap_or(&serde_json::Value::Null);
                                    let s = if v.is_null() {
                                        "".to_string()
                                    } else {
                                        v.to_string()
                                    };
                                    // 更新宽度
                                    let idx = headers.iter().position(|x| *x == *h).unwrap();
                                    widths[idx] = widths[idx].max(s.len());
                                    s
                                })
                                .collect()
                            )
                        } else {
                            None
                        }
                    })
                    .collect();
                
                // 打印表头
                for (i, h) in headers.iter().enumerate() {
                    print!("{:<width$}", h, width = widths[i]);
                    if i < headers.len() - 1 {
                        print!(" | ");
                    }
                }
                println!();
                
                // 打印分隔线
                for (i, w) in widths.iter().enumerate() {
                    print!("{}", "-".repeat(*w));
                    if i < widths.len() - 1 {
                        print!("-+-");
                    }
                }
                println!();
                
                // 打印数据行
                for row in &rows {
                    for (i, cell) in row.iter().enumerate() {
                        print!("{:<width$}", cell, width = widths[i]);
                        if i < row.len() - 1 {
                            print!(" | ");
                        }
                    }
                    println!();
                }
            }
        }
    } else {
        // 单个对象，输出为键值对
        let json_value = serde_json::to_value(data)?;
        if let Some(obj) = json_value.as_object() {
            for (k, v) in obj {
                println!("{}: {}", k, v);
            }
        }
    }
    
    Ok(())
}

/// 输出为 CSV
fn output_csv<T: Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    
    let json_value = serde_json::to_value(data)?;
    
    if let Some(array) = json_value.as_array() {
        // 写入表头（从第一个元素）
        if let Some(first) = array.first() {
            if let Some(obj) = first.as_object() {
                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                wtr.write_record(&headers)?;
            }
        }
        
        for item in array {
            if let Some(obj) = item.as_object() {
                let values: Vec<String> = obj.values()
                    .map(|v| {
                        if v.is_null() {
                            "".to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    .collect();
                wtr.write_record(&values)?;
            }
        }
    }
    
    wtr.flush()?;
    Ok(())
}
