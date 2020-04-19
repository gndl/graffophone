#ifndef __OCAML_GRAMOTOR_GRAMOTOR_STUB_H_
#define __OCAML_GRAMOTOR_GRAMOTOR_STUB_H_

#include <caml/mlvalues.h>

typedef struct gramotor Gramotor;

/* Accessing the Gramotor part of a Caml custom block */
#define Gramotor_val(v) (*((Gramotor**) Data_custom_val(v)))

#endif /*__OCAML_GRAMOTOR_GRAMOTOR_STUB_H_*/
