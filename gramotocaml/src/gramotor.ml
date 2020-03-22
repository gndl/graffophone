type t

external create: unit -> t = "gramotor_create"
let create()=
  try Result.ok (create()) with Failure msg ->Result.error msg


external new_session: t -> unit = "gramotor_new_session"

