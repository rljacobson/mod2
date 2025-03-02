(* This is an example of a WL script illustrating its features. *)

(*
Here is a custom symbolic operator. It has no intrinsic built-in meaning yet.
It takes two arguments. We want to declare it to be associative. (We are
glossing over some subtleties about "flat" versus "associative" here.)
*)

CirclePlus[a, b]
(* Displays as `a ⊕ b`, but we won't use that syntax. *)
SetAttributes[CirclePlus, Flat]

CirclePlus[a, CirclePlus[b, c]] == CirclePlus[CirclePlus[a, b], c]
(* True *)
