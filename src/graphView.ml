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

module Bus = EventBus
module Tkr = Talker
module GTkr = GTalker
module GMC = GMixingConsole

type property_t = {
	mutable start : float;
	mutable thickness : float;
	mutable count : int
}

let buildGTalker tkr gTalkers canvas =
    let gTkr = GTkr.make tkr canvas in
		gTkr#draw();
    gTalkers := (tkr#getId, gTkr) :: !gTalkers;
		gTkr

	
let provideGTalker tkr gTalkers canvas =
	try (false, L.assoc tkr#getId !gTalkers)
	with Not_found -> (true, buildGTalker tkr gTalkers canvas)


let provideProperty n propertys = try L.assoc n !propertys
	with Not_found -> (
		let prop = {start = 0.; thickness = 0.; count = 0} in
		propertys := (n, prop)::!propertys;
		prop)


let marge = 50.
let vPad = 5.

let columnLayout x0 y0 gTalkers columnsPropertys =

	(* define columns thickness *)
	L.iter gTalkers ~f:(fun (_, gTkr) ->
		let w = gTkr#getWidth in

		let colProp = provideProperty gTkr#getColumn columnsPropertys in

		if colProp.thickness < w then colProp.thickness <- w;
	);

	(* sort columns propertys from rigth to left (in increasing column number) *)
	let columnsProps = L.sort !columnsPropertys
		~cmp:(fun (n1, _) (n2, _) -> n1 - n2)
	in

	(* define graph width and row count *)
	let (w, rowCount) = L.fold_left columnsProps ~init:(0., 0)
		~f:(fun (w, rowCount) (_, colProp) ->
			(w +. colProp.thickness +. marge, max rowCount colProp.count)
		)
	in

	let prevRowsY = A.make rowCount 0. in

	(* position GTalkers *)
	let (_, h) = L.fold_left columnsProps ~init:(w, 0.)
		~f:(fun (prevX, hMax) (colNbr, colProp) ->

		let colGTkrs = L.fold_left gTalkers ~init:[] ~f:(fun gTkrs (_, gTkr) ->
			if gTkr#getColumn = colNbr then gTkr::gTkrs else gTkrs)
		in

		let sortedColGTkrs = L.sort ~cmp:(fun gt1 gt2 ->
			gt1#getRow - gt2#getRow ) colGTkrs in

		let (_, h) = L.fold_left sortedColGTkrs ~init:(-1, 0.)
			~f:(fun (prevRow, prevBottom) gTkr ->

			let row = gTkr#getRow in
			let x = prevX -. ((gTkr#getWidth +. colProp.thickness) *. 0.5) in

			let y = if row - prevRow > 1 then
					max prevBottom prevRowsY.(gTkr#getDependentRow)
				else prevBottom
			in

			gTkr#move (x +. x0) (y +. y0);

			prevRowsY.(row) <- y;
			(row, y +. gTkr#getHeight +. vPad)
		)
		in
		(prevX -. colProp.thickness -. marge, max h hMax)
	)
	in
	(w, h)


let makeGTalkers row column columnsPropertys gTalkers canvas talker =

	(* create GTalkers and define there row and column *)
	let rec mkGtkrs row column tkr =

		if not tkr#isHidden then (
  		let (newGTalker, gTkr) = provideGTalker tkr gTalkers canvas
  		in
  		let talks = tkr#getTalks in
  		let talksCount = L.length talks in
  
  		if newGTalker || (gTkr#getColumn < column && talksCount = 0) then (
  
  			gTkr#setColumn column;
  
  			let colProp = provideProperty column columnsPropertys in
  			let gTkrRow = max row colProp.count in
  			gTkr#setRow gTkrRow;
  			colProp.count <- gTkrRow + 1;
				
				if newGTalker then (
					gTkr#setDependentRow row;
				);
  
  			let depRow = gTkrRow - talksCount / 2 in
  			let depColumn = column + 1 in
  
  			L.iter talks ~f:(fun talk ->
  				mkGtkrs depRow depColumn (Ear.getTalkTalker talk));
  		);
    )
	in
	mkGtkrs row column talker


let createGraph canvas =

	let gTalkers = ref [] in
	let columnsPropertys = ref [] in
	try
	let createMixingConsole row mixCon =

		let gMc = GMC.makeAt mixCon row 0 canvas in
		gMc#draw();
    gTalkers := (mixCon#getId, gMc#base) :: !gTalkers;

	(* create GTalkers by covering talkers for each track *)
		L.iter mixCon#getTracks ~f:(fun track ->
(*			ignore(L.fold_left track#getTalks ~init:(row, 1) ~f:createGTalkers);*)
			L.iter track#getTalks ~f:(fun talk ->
				makeGTalkers row 1 columnsPropertys gTalkers canvas (Ear.getTalkTalker talk)
			);
		);
		
		L.iter mixCon#getTalks ~f:(fun talk ->
			makeGTalkers row 1 columnsPropertys gTalkers canvas (Ear.getTalkTalker talk)
		);
		gMc
	in

	(* create graph by covering mixing consoles *)
	let mixConsColProp = provideProperty 0 columnsPropertys in

	let (rowCount, gMcs) = L.fold_left (Session.getMixingConsoles()) ~init:(0, [])
		~f:(fun (row, gMcs) (_, mixCon) ->
			let gMc = createMixingConsole row mixCon in
			(row + 1, gMc::gMcs))
	in

	mixConsColProp.count <- rowCount;

	(* position GTalkers *)
	let (w, h) = columnLayout 0. 0. !gTalkers columnsPropertys in

	(*********** SANDBOX ***********)
	(* create unused GTalkers *)
(*	let (uW, uH) = positionUnusedTalkers marge (h +. marge) gTalkers canvas in*)

	(* list the unused talkers e.g not in the gTalkers list *)
	let unusedTalkers = L.fold_left (Session.getTalkers()) ~init:[]
		~f:(fun unusedTalkers (id, tkr) ->
			if tkr#isHidden || L.mem_assoc id !gTalkers then unusedTalkers
			else (tkr, ref 0)::unusedTalkers
		)
	in
	(* define the unused talkers reference count *)
	L.iter unusedTalkers ~f:(fun (tkr, _) ->
		L.iter tkr#getTalks ~f:(fun talk ->

			let depTkr = Ear.getTalkTalker talk in

  		try
  			let refCount = L.assq depTkr unusedTalkers in
  			incr refCount;
  		with Not_found -> ()
		)
	);
	let sandboxColumnsPropertys = ref [] in

	let rootUnusedTalkers = L.fold_left unusedTalkers ~init:[]
		~f:(fun l (tkr, refCount) -> if !refCount = 0 then tkr::l else l)
	in
	(* sort the root unused talkers in decreasing order
		 in order to have the newest talker at the top of the sandbox *)
	let rootUnusedTalkers = L.sort rootUnusedTalkers
		~cmp:(fun tkr1 tkr2 -> tkr2#getId - tkr1#getId)
	in

	L.iter rootUnusedTalkers ~f:(makeGTalkers 0 0 sandboxColumnsPropertys gTalkers canvas);

	let unusedGTalkers = L.map unusedTalkers
		~f:(fun (tkr, _) -> (tkr#getId, L.assoc tkr#getId !gTalkers))
	in

	(* position unused GTalkers under used GTalkers e.g the sandbox zone *)
	let (sdW, sdH) =
		columnLayout marge (h +. marge) unusedGTalkers sandboxColumnsPropertys
	in

	(* add GTracks in GTalkers list for connection and action *)
	L.iter gMcs ~f:(fun gMc ->
		L.iter gMc#getGTracks ~f:(fun gTrk ->
			gTalkers := (gTrk#getTalker#getId, gTrk#base) :: !gTalkers
		)
	);

	(* draw connections *)
	L.iter !gTalkers ~f:(fun (_, gTkr) -> gTkr#drawConnections !gTalkers canvas);

	canvas#set_scroll_region ~x1:0. ~y1:(-.vPad)
		~x2:((max w sdW) +. vPad) ~y2:(h +. marge +. sdH +. vPad);
(*		~x2:(w +. vPad) ~y2:(h +. vPad);*)

	!gTalkers

	with exc -> (
		traceMagenta(Printexc.to_string exc);
		traceYellow(Printexc.get_backtrace());
		!gTalkers
	)



class c (pGraphCtrl : GraphControler.c) = object (self)

	val mutable mGraphCanvas = GnoCanvas.canvas ~aa:false ~border_width:5 ~show:true ()
	val mutable mGTalkers = []

	initializer
		Bus.addObserver self#observe;

		mGraphCanvas#set_pixels_per_unit 1.0;
		mGraphCanvas#set_center_scroll_region false;
		mGraphCanvas#scroll_to ~x:0 ~y:0;

    let cs = mGraphCanvas#misc#style in
		cs#set_bg Style.background;
    mGraphCanvas#misc#set_style cs;


	method getGraphCanvas = mGraphCanvas
	

	method build =
		L.iter mGraphCanvas#root#get_items ~f:(fun item -> item#destroy ());

		mGTalkers <- createGraph mGraphCanvas;

		L.iter mGTalkers ~f:(fun (id, gTkr) -> gTkr#setActions pGraphCtrl);

(*
	method addNewTalker talker = trace "graphView#addNewTalker"; self#build
*)


(* observer methods *)
	method observe ev =	
		try match ev with
		| Bus.Session -> self#build
		| Bus.TalkerChanged -> self#build
  	| Bus.TalkerSelected tkrId -> (L.assoc tkrId mGTalkers)#select
  	| Bus.TalkerUnselected tkrId -> (L.assoc tkrId mGTalkers)#unselect
		| Bus.EarSelected (tkrId, idx) -> (L.assoc tkrId mGTalkers)#selectEar idx
		| Bus.EarUnselected (tkrId, idx) -> (L.assoc tkrId mGTalkers)#unselectEar idx
		| Bus.VoiceSelected (tkrId, idx) -> (L.assoc tkrId mGTalkers)#selectVoice idx
		| Bus.VoiceUnselected (tkrId, idx) -> (L.assoc tkrId mGTalkers)#unselectVoice idx
		| Bus.NewTalker -> self#build (*self#addNewTalker pGraphCtrl#getNewTalker*)
		| _ -> ()
		with Not_found -> ()

end


let gridLayout x0 y0 gTalkers rowsPropertys columnsPropertys =

	(* define rows and columns thickness *)
	L.iter gTalkers ~f:(fun (_, gTkr) ->
		let w = gTkr#getWidth in
		let h = gTkr#getHeight in

		let rowProp = provideProperty gTkr#getRow rowsPropertys in
		let colProp = provideProperty gTkr#getColumn columnsPropertys in

		if rowProp.thickness < h then rowProp.thickness <- h;
		if colProp.thickness < w then colProp.thickness <- w;
	);

	(* define rows start (y) *)
	let rowsProps = L.sort ~cmp:(fun (n1, _) (n2, _) -> n1 - n2) !rowsPropertys in

	let h = L.fold_left rowsProps ~init:0. ~f:(fun start (n, prop) ->
		prop.start <- start;
		start +. prop.thickness +. vPad
	) in

	(* define columns start (x) *)
	let columnsProps = L.sort ~cmp:(fun (n1, _) (n2, _) -> n1 - n2) !columnsPropertys in

	let w = L.fold_right columnsProps ~init:0. ~f:(fun (n, prop) start ->
		prop.start <- start;
		start +. prop.thickness +. marge
	) in

	(* position GTalkers *)
	L.iter gTalkers ~f:(fun (_, gTkr) ->
		let rowProp = provideProperty gTkr#getRow rowsPropertys in
		let colProp = provideProperty gTkr#getColumn columnsPropertys in

		let x = colProp.start +. (colProp.thickness -. gTkr#getWidth) /. 2. in
		let y = rowProp.start (*+. (rowProp.thickness -. gTkr#getHeight) /. 2.*) in

		gTkr#move (x +. x0) (y +. y0);
	);
	(w, h)


let makeUnusedGTalkers x0 y0 gTalkers canvas =

	let rowsPropertys = ref [] in
	let columnsPropertys = ref [] in

	(* list the unused talkers e.g not in the gTalkers list *)
	let unusedTalkers = L.fold_left (Session.getTalkers()) ~init:[]
		~f:(fun unusedTalkers (id, tkr) ->
			if L.mem_assoc id !gTalkers then unusedTalkers
			else (tkr, ref 0)::unusedTalkers
		)
	in
	L.iter unusedTalkers ~f:(fun (tkr, _) ->
		L.iter tkr#getTalks ~f:(fun talk ->
  		
			let depTkr = Ear.getTalkTalker talk in
  		
  		try
  			let refCount = L.assq depTkr unusedTalkers in
  			incr refCount;
  		with Not_found -> ()
		)
	);
	(* sort the talkers in decreasing order
	let unusedTalkers = L.sort unusedTalkers
		~cmp:(fun (tkr1, _) (tkr2, _) -> tkr2#getId - tkr1#getId)
	in
	*)
	let rec createGTalkers row column unusedGTalkers = function
		| [] -> unusedGTalkers
		| (tkr, _)::tl ->
			let gTkr = buildGTalker tkr gTalkers canvas in

			let rowProp = provideProperty row rowsPropertys in
			let colProp = provideProperty column columnsPropertys in

			rowProp.count <- column + 1;
			colProp.count <- row + 1;

			gTkr#setRow row; gTkr#setColumn column;

			let (nextRow, nextColumn) = if column < 5 then (row, (column + 1))
				else ((row + 1), 0)
			in
			createGTalkers nextRow nextColumn ((tkr#getId, gTkr)::unusedGTalkers) tl
	in
	let unusedGTalkers = createGTalkers 0 0 [] unusedTalkers in

	gridLayout x0 y0 unusedGTalkers rowsPropertys columnsPropertys


	(* create GTalkers and define there row and column *)(*
	let rec createGTalkers (row, column) talk =
		let tkr = Ear.getTalkTalker talk in

		let (newGTalker, gTkr) = provideGTalker tkr gTalkers canvas
		in
		let talks = tkr#getTalks in
		let talksCount = L.length talks in

		if newGTalker || (gTkr#getColumn < column && talksCount = 0) then (

			gTkr#setColumn column;

			let colProp = provideProperty column columnsPropertys in
			let gTkrRow = max row colProp.count in
			gTkr#setRow gTkrRow;
			colProp.count <- gTkrRow + 1;

			let firstTalkRow = gTkrRow - talksCount / 2 in
			ignore(L.fold_left talks ~init:(firstTalkRow, column + 1) ~f:createGTalkers);
		);
		(row, column)
	in
*)
