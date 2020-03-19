type t

external create: unit -> t = "gramotor_create"
external new_session: t -> unit = "gramotor_new_session"

