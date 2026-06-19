use mlua::{Function, Lua, Result, Value, Variadic};

pub(crate) fn run_script(script: &str) -> Result<Vec<String>> {
    let lua = Lua::new();
    let ds = lua.create_table()?;
    let output = lua.create_table()?;
    let print_output = output.clone();

    ds.set(
        "print",
        lua.create_function(move |_, values: Variadic<Value>| {
            let line = values
                .into_iter()
                .map(lua_value_to_string)
                .collect::<Vec<_>>()
                .join("\t");
            print_output.push(line)
        })?,
    )?;

    let wrapped_script = format!("{script}\nreturn main");

    let main: Function = lua.load(&wrapped_script).eval()?;
    main.call::<()>(ds)?;

    output.sequence_values().collect()
}

fn lua_value_to_string(value: Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.to_string_lossy(),
        _ => format!("{value:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::run_script;

    #[test]
    fn runs_main_with_ds_api() {
        let output = run_script(
            r#"
            local function main(ds)
                assert(ds ~= nil)
            end
            "#,
        )
        .unwrap();

        assert_eq!(output, Vec::<String>::new());
    }

    #[test]
    fn ds_print_captures_output() {
        let output = run_script(
            r#"
            local function main(ds)
                ds.print("money", 1024, true, nil)
            end
            "#,
        )
        .unwrap();

        assert_eq!(output, vec!["money\t1024\ttrue\tnil"]);
    }

    #[test]
    fn requires_main_function() {
        assert!(run_script("").is_err());
    }
}
