#[cfg(test)]
mod term {
    use std::any::{Any, TypeId};

    use itertools::Itertools;
    use rustls::internal::msgs::handshake::SessionID;

    use crate::tls::fn_impl::{fn_client_hello, fn_hmac256, fn_hmac256_new_key, fn_session_id};
    use crate::tls::{REGISTERED_FN, REGISTERED_TYPES, FnError};
    use crate::{
        term::{Signature, Term},
        trace::TraceContext,
    };

    fn example_op_c(a: &u8) -> Result<u16, FnError> {
       Ok( (a + 1) as u16)
    }

    #[test]
    fn example() {
        let mut sig = Signature::default();

        let hmac256_new_key = sig.new_function(&fn_hmac256_new_key);
        let hmac256 = sig.new_function(&fn_hmac256);
        let _client_hello = sig.new_function(&fn_client_hello);

        let data = "hello".as_bytes().to_vec();

        println!("TypeId of vec array {:?}", data.type_id());

        let variable = sig.new_var::<Vec<u8>>((0, 0));

        let generated_term = Term::Application(
            hmac256,
            vec![
                Term::Application(hmac256_new_key, vec![]),
                Term::Variable(variable),
            ],
        );

        println!("{}", generated_term);
        let mut context = TraceContext::new();
        context.add_variable((0, 0), Box::new(data));

        println!(
            "{:?}",
            generated_term
                .evaluate(&context)
                .as_ref()
                .unwrap()
                .downcast_ref::<Vec<u8>>()
        );
    }

    #[test]
    fn playground() {
        let mut sig = Signature::default();

        let example = sig.new_function(&example_op_c);
        let example1 = sig.new_function(&example_op_c);

        let var_data = fn_session_id();

        let k = sig.new_var::<SessionID>((0, 0));

        println!("vec {:?}", TypeId::of::<Vec<u8>>());
        println!("vec {:?}", TypeId::of::<Vec<u16>>());

        println!("{:?}", TypeId::of::<SessionID>());
        println!("{:?}", var_data.type_id());

        let func = example.clone();
        let dynamic_fn = func.dynamic_fn();
        println!(
            "{:?}",
            dynamic_fn(&vec![Box::new(1u8)]).unwrap()
                .downcast_ref::<u16>()
                .unwrap()
        );
        println!("{}", example.shape());

        let constructed_term = Term::Application(
            example1.clone(),
            vec![
                Term::Application(
                    example1.clone(),
                    vec![
                        Term::Application(
                            example1.clone(),
                            vec![
                                Term::Application(example1.clone(), vec![]),
                                Term::Variable(k.clone()),
                            ],
                        ),
                        Term::Variable(k.clone()),
                    ],
                ),
                Term::Application(
                    example1.clone(),
                    vec![
                        Term::Application(
                            example1.clone(),
                            vec![
                                Term::Variable(k.clone()),
                                Term::Application(example.clone(), vec![]),
                            ],
                        ),
                        Term::Variable(k.clone()),
                    ],
                ),
            ],
        );

        println!("{}", constructed_term);
    }

    #[test]
    fn test_static_functions() {
        println!("{}", REGISTERED_FN.iter().map(|tuple| tuple.0).join("\n"));
        println!(
            "{}",
            REGISTERED_TYPES
                .iter()
                .map(|tuple| tuple.0.to_string())
                .join("\n")
        );
    }
}
