#[macro_use]
extern crate ocaml;

extern crate failure;
extern crate gramotor;

use std::mem;

use ocaml::Value;

use gramotor::gramotor::Gramotor;

extern "C" fn finalize(value: ocaml::core::Value) {
    let handle = Value(value);
    let ptr = handle.custom_ptr_val_mut::<Gramotor>();
    mem::drop(ptr);
    println!("Finalize Gramotor");
}

caml!(gramotor_create() {
    let mut gramotor = Gramotor::new();
    let ptr = &mut gramotor as *mut Gramotor;
    mem::forget(gramotor);
    Value::alloc_custom(ptr, finalize)
});

caml!(gramotor_new_session(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    let _ = motor.new_session();
    Value::unit()
});

caml!(gramotor_init_session(handle, session_description) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    let sd = ocaml::Str::from(session_description.clone());
    let _ = motor.init_session(sd.as_str().to_string());
    Value::unit()
});

caml!(gramotor_start(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    match motor.start() {
        Ok(()) => Value::unit(),
        Err(e)=>{ocaml::runtime::failwith(format!("{}", e)); Value::unit()}
    }
});
caml!(gramotor_play(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    match motor.play() {
        Ok(()) => Value::unit(),
        Err(e)=>{ocaml::runtime::failwith(format!("{}", e)); Value::unit()}
    }
});
caml!(gramotor_pause(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    match motor.pause() {
        Ok(()) => Value::unit(),
        Err(e)=>{ocaml::runtime::failwith(format!("{}", e)); Value::unit()}
    }
});
caml!(gramotor_stop(handle) {
    let motor = &mut *handle.custom_ptr_val_mut::<Gramotor>();
    match motor.stop() {
        Ok(()) => Value::unit(),
        Err(e)=>{ocaml::runtime::failwith(format!("{}", e)); Value::unit()}
    }
});
