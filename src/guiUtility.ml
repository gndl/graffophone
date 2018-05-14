(* 
 * Copyright (C) 2015 Gaëtan Dubreil
 *
 *  All rights reserved.This file is distributed under the terms of the
 *  GNU General Public License version 3.0.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 *)

open Usual

let all_files () =
  let f = GFile.filter ~name:"All" () in
  f#add_pattern "*" ;
  f

let is_string_prefix s1 s2 =
  let l1 = S.length s1 in
  let l2 = S.length s2 in
  l1 <= l2 && s1 = S.sub s2 0 l1

let image_filter () =
  let f = GFile.filter ~name:"Images" () in
  f#add_custom [ `MIME_TYPE ]
    (fun info ->
       let mime = List.assoc `MIME_TYPE info in
       is_string_prefix "image/" mime) ;
  f

let text_filter () = 
  GFile.filter 
    ~name:"Caml source code" 
    ~patterns:[ "*.ml"; "*.mli"; "*.mly"; "*.mll" ] ()

let sessionFilter () = 
  GFile.filter 
    ~name:"Session" 
    ~patterns:[ "*.es"] ()

let audioFilter () = 
  GFile.filter 
    ~name:"Audio" 
    ~patterns:[ "*.wav"; "*.ogg"; "*.flac"; "*.m3u"; "*.pls"] ()

let openFile ?filter ?parent () =
  let dialog = GWindow.file_chooser_dialog 
      ~action:`OPEN
      (*      ~title:"Open File"*)
      ?parent () in
  dialog#add_button_stock `CANCEL `CANCEL ;
  dialog#add_select_button_stock `OPEN `OPEN ;

  ignore(match filter with Some fltr -> dialog#add_filter (fltr ()) | None -> ());

  let res = match dialog#run () with
    | `OPEN -> dialog#filename
    | `DELETE_EVENT | `CANCEL -> None
  in
  dialog#destroy (); res

let saveFile ?parent filter =
  let dialog = GWindow.file_chooser_dialog 
      ~action:`SAVE
      (*      ~title:"Open File"*)
      ?parent () in
  dialog#add_button_stock `CANCEL `CANCEL ;
  dialog#add_select_button_stock `SAVE `SAVE ;
  dialog#add_filter (filter ()) ;
  let res = match dialog#run () with
    | `SAVE -> dialog#filename
    | `DELETE_EVENT | `CANCEL -> None
  in
  dialog#destroy (); res


let aboutDialog() =

  let dialog = new GraffophoneGui.aboutDialog() in

  let _ = dialog#toplevel#connect#response(fun _ -> dialog#toplevel#destroy()) in
  let _ = dialog#toplevel#connect#close(fun() -> dialog#toplevel#destroy()) in

  dialog#toplevel#show()


let dialogStringEntry initValue callback =

  let dialog = new GraffophoneGui.stringEntryDialog() in

  dialog#entryStringEntry#set_text initValue;

  let closeDialog() = dialog#toplevel#destroy() in

  let _ = dialog#stringEntryOkButton#connect#clicked(
      fun() ->
        callback dialog#entryStringEntry#text;
        closeDialog();
    ) in

  ignore(dialog#stringEntryCancelButton#connect#clicked closeDialog);
  ignore(dialog#toplevel#connect#close closeDialog);

  dialog#toplevel#show()

let dialogTextEntry initValue callback = dialogStringEntry initValue callback


let dialogFloatEntry initValue callback =
trace("dialogFloatEntry "^sof initValue);
  let dialog = new DialogFloatEntry.c initValue callback in

  dialog#toplevel#show()

let dialogIntEntry initValue callback =
  dialogFloatEntry (foi initValue) (fun f -> callback(iof f))
