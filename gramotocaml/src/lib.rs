#[macro_use]
extern crate ocaml;

extern crate gramotor;

use std::mem;

use gramotor::Gramotor;

extern "C" fn finalize(value: ocaml::core::Value) {
    let handle = ocaml::Value(value);
    let ptr = handle.custom_ptr_val_mut::<Gramotor>();
    mem::drop(ptr);
    println!("Finalize");
}

caml!(gramotor_create(n) {
    let mut gramotor: Result<Gramotor, failure::Error> =Gramotor::new();
    let ptr = &mut gramotor as *mut Gramotor;
    mem::forget(gramotor);
    ocaml::Value::alloc_custom(ptr, finalize)
});

caml!(gramotor_new_session(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    motor.new_session();
    ocaml::Value::unit()
});
