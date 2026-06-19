use std::{cell::RefCell, rc::Rc};

use rhai::{Dynamic, Engine, EvalAltResult, Scope};

#[derive(Clone)]
struct Ds {
    output: Rc<RefCell<Vec<String>>>,
}

impl Ds {
    fn print(&mut self, value: Dynamic) {
        self.output.borrow_mut().push(dynamic_to_string(&value));
    }
}

pub(crate) fn run_script(script: &str) -> Result<Vec<String>, Box<EvalAltResult>> {
    let output = Rc::new(RefCell::new(Vec::new()));
    let ds = Ds {
        output: Rc::clone(&output),
    };
    let mut engine = Engine::new();
    engine.register_type_with_name::<Ds>("Ds");
    engine.register_fn("ds_print", Ds::print);

    let ast = engine.compile(script)?;
    engine.call_fn::<()>(&mut Scope::new(), &ast, "main", (ds,))?;

    Ok(output.borrow().clone())
}

fn dynamic_to_string(value: &Dynamic) -> String {
    if value.is::<()>() {
        "()".to_string()
    } else if let Some(value) = value.clone().try_cast::<String>() {
        value
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::run_script;

    #[test]
    fn runs_main_with_ds_api() {
        let output = run_script(
            r#"
            fn main(ds) {
                if ds == () {
                    throw "missing ds";
                }
            }
            "#,
        )
        .unwrap();

        assert_eq!(output, Vec::<String>::new());
    }

    #[test]
    fn ds_print_captures_output() {
        let output = run_script(
            r#"
            fn main(ds) {
                ds_print(ds, "money");
                ds_print(ds, 1024);
                ds_print(ds, true);
                ds_print(ds, ());
            }
            "#,
        )
        .unwrap();

        assert_eq!(output, vec!["money", "1024", "true", "()"]);
    }

    #[test]
    fn requires_main_function() {
        assert!(run_script("").is_err());
    }
}
