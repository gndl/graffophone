#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/fail.h>
#include <caml/callback.h>
#include <caml/custom.h>
#include <caml/threads.h>

#include "gramotor.h"

#include "gramotor_stub.h"

static void custom_finalize_gramotor (value v)
{
  Gramotor * m = Gramotor_val(v);

  gramotor_gramotor_drop(m);
}

// Encapsulation of opaque gramotor handles (of type GramotorGramotor) as Caml custom blocks.
static struct custom_operations gramotor_gramotor_ops =
  {
   "GRAMOTOR/OCAMLinterface/" OCAML_GRAMOTOR_VERSION "/gramotor",
   custom_finalize_gramotor,
   custom_compare_default,
   custom_hash_default,
   custom_serialize_default,
   custom_deserialize_default
  };

value caml_gramotor_gramotor_new (value unit)
{
  CAMLparam1 (unit);
  CAMLlocal1(ans);

//  caml_release_runtime_system();
  Gramotor* m = gramotor_gramotor_new();
//  caml_acquire_runtime_system();

  if(m == NULL) caml_failwith("Gramotor.Gramotor initialization failed");

  ans = caml_alloc_custom(&gramotor_gramotor_ops, sizeof(Gramotor*), 0, 1);

  Gramotor_val(ans) = gramotor;

  CAMLreturn (ans);
}

value caml_gramotor_gramotor_play(value v_gramotor)
{
  CAMLparam1 (v_gramotor);

  gramotor_gramotor_play(Gramotor_val(v_gramotor));
  CAMLreturn (Val_unit);
}
