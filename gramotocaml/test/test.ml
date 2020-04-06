let gsr = {|
Sinusoidal 1#Sinusoidal_1 
> frequence 440
> phase 0

track 2#track_2
> I 1#Sinusoidal_1:O
> gain 1

mixer 5#mixer_5
> volume 1
> track 2#track_2
|}

let () =
print_endline "Gramotor.create()";
  (match Gramotor.create() with
        | Ok gramotor -> (
print_endline "Gc.full_major ()";
              Gc.full_major (); Gc.full_major ();
print_endline "Gramotor.init_session";
            Gramotor.init_session gramotor gsr;
print_endline "Gc.full_major ()";
              Gc.full_major (); Gc.full_major ();
print_endline "Gramotor.play";
            match Gramotor.play gramotor with
            Ok()->(
              Gc.full_major (); Gc.full_major ();
              "OK!"
            )
            | Error msg -> msg
          )
            | Error msg -> msg
  )         |> print_endline
          
