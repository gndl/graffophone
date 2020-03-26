type t

val create: unit -> (t, string) result
val new_session: t -> unit
val init_session: t -> string -> unit

