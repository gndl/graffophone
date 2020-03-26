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

module SF = SampleFormat
module Bus = EventBus

type order = Pause | Stop | None

class c =
  object (self)
    val mutable mState = State.Stopped
    val mutable mOrder = None
    val mPauseLock = Mutex.create()
    val mSynchronizationLock = Mutex.create()
    val mutable mSynchronizationRequest = false
    val mutable mStartTick = 0
    val mutable mEndTick = 0
    val mutable mControlKeyPressed = false
    val mutable mShiftKeyPressed = false
    val mutable mAltKeyPressed = false
      val mutable mGramotor : Gramotor.t option = None


    val mCurve = new CurveControler.c
    val mGraph = new GraphControler.c

    method curve = mCurve
    method graph = mGraph


    method init() =

      (* generate talkers menu *)
      let talkersInfos = Factory.getTalkersInfos() in

      let categorys = L.fold_left talkersInfos ~init:[]
          ~f:(fun cats (_, category) ->
              if L.mem category ~set:cats then cats else category::cats)
      in
      let categorys = L.sort ~cmp:String.compare categorys in

      let talkersRange = L.map categorys ~f:(fun category ->

          let catKinds = L.fold_left talkersInfos ~init:[]
              ~f:(fun l (knd, cat) -> if cat = category then knd::l else l)
          in
          let catKinds = L.sort catKinds ~cmp:String.compare
          in
          (category, catKinds)
        )
      in
      Bus.(notify(TalkersRange talkersRange));

      self#newSession();

    method getState = mState
    method setOrder o = mOrder <- o;

    method getStartTick = mStartTick

    method setStartTick t =
      if mEndTick = mStartTick then (
        mStartTick <- t;
        mEndTick <- t;
        Bus.(notify(Bus.Tick t));
      )
      else (
        mStartTick <- t;
        Bus.(notify(TimeRange(t, mEndTick)));
      );


    method getEndTick = mEndTick

    method setEndTick t =
      mEndTick <- t;
      Bus.(notify(TimeRange(mStartTick, t)));



    method newSession() =
      let () = match Gramotor.create() with
      | Ok gramotor -> mGramotor <- Some(gramotor)
      | Error msg -> traceRed msg
      in

      let track = new Track.c in
      let output = (new PlaybackOutput.c() :> Output.c) in
      let mixingConsole = new MixingConsole.c ~tracks:[track] ~outputs:[output] ()
      in
      ignore(Session.make ~tracks:[(0, track)]
               ~mixingConsoles:[(0, mixingConsole)] ~outputs:[(0, output)] ());

      Bus.(notify Session);


    method openSession sessionName =
      ignore(Session.load sessionName);
      Bus.(notify Session);


    method saveSession() = Session.save (Session.getInstance())


    method saveSessionAs filename =
      ignore(Session.saveAs filename (Session.getInstance()))


    method play (_:int) =
      ignore( match mState with
          | State.Playing -> Mutex.lock mPauseLock; mOrder <- Pause
          | State.Paused -> Mutex.unlock mPauseLock
          | State.Stopped -> ignore(Thread.create self#threadPlay())
        );
      Thread.yield()

    method pause (_:int) =
      ignore( match mState with
          | State.Playing -> Mutex.lock mPauseLock; mOrder <- Pause
          | State.Paused -> Mutex.unlock mPauseLock
          | State.Stopped -> ()
        );
      Thread.yield()

    method stop (_:int) =
      ignore( match mState with
          | State.Playing -> mOrder <- Stop
          | State.Paused -> mOrder <- Stop; Mutex.unlock mPauseLock
          | State.Stopped -> ()
        );
      Thread.yield()

    method changeVolume volumePercent =
      Bus.(notify(Volume volumePercent));


    method addNewTalker kind =
      let tkr = Factory.makeTalker kind in
      Session.addTalker tkr;
      Bus.(notify NewTalker);
      mGraph#addNewTalker tkr;(**)


    method setControlKeyPressed v = mControlKeyPressed <- v; mGraph#setControlKeyPressed v
    method setShiftKeyPressed v = mShiftKeyPressed <- v; mGraph#setShiftKeyPressed v
    method setAltKeyPressed v = mAltKeyPressed <- v; mGraph#setAltKeyPressed v


    method synchronize (f : (unit -> unit)) =

      match mState with
      | State.Playing ->
        mSynchronizationRequest <- true;
        Mutex.lock mSynchronizationLock;
        mSynchronizationRequest <- false;
        f();
        Mutex.unlock mSynchronizationLock;
      | _ -> f()


    method private threadPlay() =

      Mutex.lock mSynchronizationLock;
      mState <- State.Playing;
      Bus.(notify(State mState));

      let sd = SF.chunkSize in
      let buf = Array.make sd 0. in
      trace("Open output at "^sof (Sys.time()));
      List.iter (fun (_, mc) -> mc#openOutput) (Session.getMixingConsoles());


      let playChunk t =
        let (d, continu) = ListLabels.fold_left ~init:(sd, false)
            (Session.getMixingConsoles())
            ~f:(fun (d, c) (_, mc) ->
                if mc#isProductive then (
                  try (*trace("comOut "^soi t^" "^soi d);*)
                    let nd = mc#comeOut t buf d in
                    (nd, true)
                  with Voice.End -> mc#setProductive false; (d, c)
                )
                else (d, c)
              );
        in (t + d, continu)
      in

      let rec eventLoop t =

        if mSynchronizationRequest then (
          Mutex.unlock mSynchronizationLock;
          Thread.yield();
          Mutex.lock mSynchronizationLock;
        );

        let (nt, continu) = match mOrder with
          | None -> playChunk t
          | Pause -> trace"state pause";
            mOrder <- None;
            mState <- State.Paused;
            Bus.(notify(State mState));
            Mutex.lock mPauseLock;
            Mutex.unlock mPauseLock;
            mState <- State.Playing;
            Bus.(notify(State mState));
            (t, true)
          | Stop -> trace"state stop";
            mOrder <- None; (t, false)
        in
        Bus.(notify(Tick nt));

        if continu then eventLoop nt
        else nt
      in
      let endTick = eventLoop mStartTick in

      mState <- State.Stopped;
      Bus.(notify(State mState));
      Bus.(notify(Tick mStartTick));
      Mutex.unlock mSynchronizationLock;

      List.iter (fun (_, mc) -> mc#closeOutput) (Session.getMixingConsoles());
      trace("Close output at "^sof (Sys.time()));

      let nbTick = endTick - mStartTick in
      trace(soi nbTick ^" sample played at "^sof (Sys.time()));
      trace("Duration : "^soi(nbTick / (60 * SF.rate))^" min "^soi((nbTick / SF.rate) mod 60)^" sec");

  end
