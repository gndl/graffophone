#[macro_use]
extern crate ocaml;

extern crate failure;
extern crate gramotor;

use std::mem;

use gramotor::gramotor::Gramotor;

extern "C" fn finalize(value: ocaml::core::Value) {
    let handle = ocaml::Value(value);
    let ptr = handle.custom_ptr_val_mut::<Gramotor>();
    mem::drop(ptr);
    println!("Finalize");
}

caml!(gramotor_create() {
    // let mut gramotor: Result<Gramotor, failure::Error> =
        match Gramotor::new(){
            Ok(mut gramotor)=>{
    let ptr = &mut gramotor as *mut Gramotor;
    mem::forget(gramotor);
                ocaml::Value::alloc_custom(ptr, finalize)
            },
            Err(e)=>{ocaml::runtime::failwith(format!("{}", e));ocaml::Value::unit()}
        }
});

caml!(gramotor_new_session(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    let _ = motor.new_session();
    ocaml::Value::unit()
});

caml!(gramotor_init_session(handle, session_description) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    let sd = ocaml::Str::from(session_description.clone());
    let _ = motor.init_session(sd.as_str());
    ocaml::Value::unit()
});
