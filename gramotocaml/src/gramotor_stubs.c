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
  OcamlGramotor * ow = OcamlGramotor_val(v);
  
  for(int i = 0; i < OCAML_GRAMOTOR_LV2_NODES_LEN; i++) {
    if(ow->nodes[i]) gramotor_node_free(ow->nodes[i]);
  }

  gramotor_gramotor_free(ow->gramotorGramotor);
  free(ow);
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

/* Allocating a Caml custom block to hold the given GramotorGramotor */
static void ocaml_gramotor_alloc_gramotor(GramotorGramotor* gramotor, value * pvalue)
{
  OcamlGramotor * ow = (OcamlGramotor *)calloc(1, sizeof(OcamlGramotor));
  ow->gramotorGramotor = gramotor;
  
  *pvalue = caml_alloc_custom(&gramotor_gramotor_ops, sizeof(OcamlGramotor*), 0, 1);

  OcamlGramotor_val(*pvalue) = ow;
}

const GramotorNode * ocaml_gramotor_get_node_by_id(OcamlGramotor * ow, int id)
{
  if(!ow->nodes[id]) {
    ow->nodes[id] = gramotor_new_uri(ow->gramotorGramotor, ocaml_gramotor_lv2_uri(id));
  }
  
  return ow->nodes[id];
}

value caml_gramotor_gramotor_new (value unit)
{
  CAMLparam1 (unit);
  CAMLlocal1(ans);

  caml_release_runtime_system();
  GramotorGramotor* m = gramotor_gramotor_new();
  caml_acquire_runtime_system();
   
  if(m == NULL) caml_failwith("Gramotor.Gramotor initialization failed");

  ocaml_gramotor_alloc_gramotor(w, &ans);
  
  CAMLreturn (ans);
}

//void gramotor_gramotor_set_option(GramotorGramotor* gramotor, const char* uri, const GramotorNode* value);
value caml_gramotor_gramotor_set_option(value v_gramotor, value v_uri, value v_value)
{
  CAMLparam3 (v_gramotor, v_uri, v_value);
  gramotor_gramotor_set_option(Gramotor_val(v_gramotor), String_val(v_uri), Node_val(v_value));
  CAMLreturn (Val_unit);
}
