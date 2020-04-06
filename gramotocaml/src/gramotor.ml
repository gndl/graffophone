type t

external create: unit -> t = "gramotor_create"
let create()=
  try Result.ok (create()) with Failure msg -> Result.error msg


external new_session: t -> unit = "gramotor_new_session"
external init_session: t -> string -> unit = "gramotor_init_session"

external start: t -> unit = "gramotor_start"
let start motor =
  try Result.ok (start motor) with Failure msg -> Result.error msg

external play: t -> unit = "gramotor_play"
let play motor =
  try Result.ok (play motor) with Failure msg -> Result.error msg

external pause: t -> unit = "gramotor_pause"
let pause motor =
  try Result.ok (pause motor) with Failure msg -> Result.error msg

external stop: t -> unit = "gramotor_stop"
let stop motor =
  try Result.ok (stop motor) with Failure msg -> Result.error msg

