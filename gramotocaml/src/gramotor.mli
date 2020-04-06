type t

val create: unit -> (t, string) result
val new_session: t -> unit
val init_session: t -> string -> unit
val start: t -> (unit, string) result
val play: t -> (unit, string) result
val pause: t -> (unit, string) result
val stop: t -> (unit, string) result

