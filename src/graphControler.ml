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

open Graffophone_plugin
open Usual

module Bus = EventBus
module Tkr = Talker

class c = object (self)

  val mutable mSelectedEar : (Tkr.c * int) option = None
  val mutable mSelectedVoice : (Tkr.c * int) option = None
  val mutable mNewTalker : Tkr.c option = None
  val mutable mSelectedTalkers : Tkr.c list = []
  val mutable mControlKeyPressed = false
  val mutable mShiftKeyPressed = false
  val mutable mAltKeyPressed = false


  method setControlKeyPressed v = mControlKeyPressed <- v
  method setShiftKeyPressed v = mShiftKeyPressed <- v
  method setAltKeyPressed v = mAltKeyPressed <- v

  method setSelectedTalker tkr =

    if mControlKeyPressed then (

      if L.memq tkr mSelectedTalkers then (
        mSelectedTalkers <- L.filter (fun t -> t != tkr) mSelectedTalkers;
        Bus.notify(Bus.TalkerUnselected tkr#getId)
      )
      else (
        mSelectedTalkers <- tkr::mSelectedTalkers;
        Bus.notify(Bus.TalkerSelected tkr#getId);
      )
    )
    else (
      if L.length mSelectedTalkers > 1 then (
        L.iter(fun tkr -> Bus.notify(Bus.TalkerUnselected tkr#getId)) mSelectedTalkers;
        mSelectedTalkers <- [tkr];
        Bus.notify(Bus.TalkerSelected tkr#getId);
      )
      else (
        if L.memq tkr mSelectedTalkers then (
          mSelectedTalkers <- [];
          Bus.notify(Bus.TalkerUnselected tkr#getId)
        )
        else (
          mSelectedTalkers <- [tkr];
          Bus.notify(Bus.TalkerSelected tkr#getId);
        )
      )
    )


  method setTalkerName (tkr:Tkr.c) v =
    tkr#setName v;
    Bus.notify Bus.TalkerChanged;


  method setTalkerValue (tkr:Tkr.c) v fly =
    tkr#setValue v;
    if not fly then Bus.notify Bus.TalkerChanged;


  method setTalkerEarToValueByIndex (tkr:Tkr.c) index value fly =
    tkr#setEarToValueByIndex index value;
    if not fly then Bus.notify Bus.TalkerChanged;


  method addTalkerEarToValueByIndex (tkr:Tkr.c) index value =
    tkr#addEarToValueByIndex index value;
    Bus.notify Bus.TalkerChanged;


  method selectEar talker index =
    match mSelectedEar with
    | Some (tkr, idx) -> (
        if talker != tkr || index <> idx then (
          mSelectedEar <- Some (talker, index);
          Bus.notify(Bus.EarSelected (talker#getId, index))
        )
        else (
          mSelectedEar <- None
        );
        Bus.notify(Bus.EarUnselected (tkr#getId, idx));
      )
    | None -> (
        match mSelectedVoice with
        | None ->
          mSelectedEar <- Some (talker, index);
          Bus.notify(Bus.EarSelected (talker#getId, index))
        | Some (tkr, idx) ->
          if tkr != talker then (
            let voice = tkr#getVoices.(idx) in
            talker#setEarToVoiceByIndex index voice;
            mSelectedVoice <- None;
            Bus.notify(Bus.VoiceUnselected (tkr#getId, idx));
            Bus.notify Bus.TalkerChanged;
          )
      )


  method selectVoice talker index =
    match mSelectedVoice with
    | Some (tkr, idx) -> (
        if talker != tkr || index <> idx then (
          mSelectedVoice <- Some (talker, index);
          Bus.notify(Bus.VoiceSelected (talker#getId, index))
        )
        else (
          mSelectedVoice <- None
        );
        Bus.notify(Bus.VoiceUnselected (tkr#getId, idx));
      )
    | None -> (
        match mSelectedEar with
        | None ->
          mSelectedVoice <- Some (talker, index);
          Bus.notify(Bus.VoiceSelected (talker#getId, index))
        | Some (tkr, idx) ->
          if tkr != talker then (
            let voice = talker#getVoices.(index) in
            tkr#setEarToVoiceByIndex idx voice;
            mSelectedEar <- None;
            Bus.notify(Bus.EarUnselected (tkr#getId, idx));
            Bus.notify Bus.TalkerChanged;
          )
      )


  method addEar (talker:Tkr.c) (rootIndex:int) =

    match mSelectedVoice with None -> ()
                            | Some (tkr, idx) ->
                              talker#addEarToVoiceByIndex rootIndex (tkr#getVoices.(idx));
                              Bus.notify Bus.TalkerChanged


  method supEar (talker:Tkr.c) (index:int) =
    talker#supEarByIndex index;
    Bus.notify Bus.TalkerChanged;


  method addNewTalker talker =

    ignore(match mSelectedVoice with None -> ()
                                   | Some (tkr, idx) -> Bus.notify(Bus.VoiceUnselected (tkr#getId, idx)));

    ignore(match mSelectedEar with None -> ()
                                 | Some (tkr, idx) -> Bus.notify(Bus.EarUnselected (tkr#getId, idx)));

    ignore(match mNewTalker with None -> ()
                               | Some tkr ->
                                 mSelectedVoice <- Some (tkr, 0);
                                 Bus.notify(Bus.VoiceSelected (tkr#getId, 0))
      );

    mNewTalker <- Some talker;


  method getNewTalker = mNewTalker

  method deleteTalker (talker:Tkr.c) = trace("Delete talker "^ talker#getName);
    Session.supTalker talker;
    Bus.notify Bus.TalkerChanged;


  method selectTalk (talk:Tkr.talk_t) =
    let tkr = Ear.getTalkTalker talk in
    let port = Ear.getTalkPort talk in
    Bus.notify (Bus.TalkSelected(tkr#getId, port));

end
