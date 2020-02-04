/*
 * Copyright (C) 2020 Gaëtan Dubreil
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
 */

use crate::curve_controler::CurveControler;
use crate::event_bus::{Notification, REventBus};
use crate::graph_controler::GraphControler;
use crate::state::State;

use std::sync::Mutex;

enum Order {
    Pause,
    Stop,
    None,
}

struct SessionControler {
    state: State,
    order: Order,
    pause_lock: Mutex<i64>,
    synchronization_lock: Mutex<i64>,
    synchronization_request: bool,
    start_tick: i64,
    end_tick: i64,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    curve: CurveControler,
    graph: GraphControler,
    bus: REventBus,
}

impl SessionControler {
    pub fn new(bus: REventBus) -> SessionControler {
        Self {
            state: State::Stopped,
            order: Order::None,
            pause_lock: Mutex::new(0),
            synchronization_lock: Mutex::new(0),
            synchronization_request: false,
            start_tick: 0,
            end_tick: 0,
            control_key_pressed: false,
            shift_key_pressed: false,
            alt_key_pressed: false,
            curve: CurveControler::new(),
            graph: GraphControler::new(),
            bus,
        }
    }

    pub fn curve<'a>(&'a self) -> &'a CurveControler {
        &self.curve
    }
    pub fn graph<'a>(&'a self) -> &'a GraphControler {
        &self.graph
    }
    pub fn state<'a>(&'a self) -> &'a State {
        &self.state
    }
    pub fn set_order(&mut self, order: Order) {
        self.order = order;
    }

    pub fn start_tick(&self) -> i64 {
        self.start_tick
    }

    pub fn set_start_tick(&mut self, t: i64) {
        if self.start_tick == self.end_tick {
            self.start_tick = t;
            self.end_tick = t;
            self.bus.notify(Notification::Tick(t));
        } else {
            self.start_tick = t;
            self.bus.notify(Notification::TimeRange(t, self.end_tick));
        }
    }

    pub fn end_tick(&self) -> i64 {
        self.end_tick
    }

    pub fn set_end_tick(&mut self, t: i64) {
        self.end_tick = t;
        self.bus.notify(Notification::TimeRange(self.start_tick, t));
    }
}
/*
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

    method newSession() =
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
*/
